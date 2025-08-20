//! [`Gamepad`] and related types.

pub(crate) mod input;
#[cfg(feature = "rumble")]
#[cfg_attr(docsrs, doc(cfg(feature = "rumble")))]
pub(crate) mod rumble;
#[cfg(feature = "sensors")]
#[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
pub(crate) mod sensors;
#[cfg(feature = "touchpad")]
#[cfg_attr(docsrs, doc(cfg(feature = "touchpad")))]
pub(crate) mod touchpad;

use alloc::string::{String, ToString as _};
#[cfg(feature = "touchpad")]
use alloc::{vec, vec::Vec};
use core::{cmp, fmt, hash};

use sdl2::{
    controller::GameController as SdlController,
    joystick::{Joystick as SdlJoystick, PowerLevel as SdlPowerLevel},
};

use crate::Error;
#[cfg(feature = "touchpad")]
use crate::TouchpadState;

/// Represents a physical game controller.
///
/// Can be obtained from [`Girl::gamepad`] or [`Girl::gamepads_connected`]
/// iterator.
///
/// # Examples
///
/// ```
/// let mut girl = girl::Girl::new()?;
/// # if girl.gamepad(0).is_some() {
/// let mut gamepad = girl.gamepad(0).unwrap();
/// # }
///
/// // check buttons, sensors, etc.
/// # Ok::<(), girl::Error>(())
/// ```
///
/// [`Girl::gamepad`]: crate::Girl::gamepad
/// [`Girl::gamepads_connected`]: crate::Girl::gamepads_connected
pub struct Gamepad {
    /// SDL2 game controller handle.
    gp: SdlController,

    /// SDL2 joystick handle.
    joy: SdlJoystick,

    /// Touchpad state for each touchpad and finger.
    #[cfg(feature = "touchpad")]
    #[cfg_attr(docsrs, doc(cfg(feature = "touchpad")))]
    touchpads: Vec<Vec<TouchpadState>>,
}

impl fmt::Debug for Gamepad {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gamepad")
            .field("gp_id", &self.gp.instance_id())
            .field("joy_id", &self.joy.instance_id())
            .finish_non_exhaustive()
    }
}

/// Displays the name of the [`Gamepad`] (or just "Gamepad" if not found), its
/// power level (if available), and its internal SDL2 instance ID.
///
/// # Examples
///
/// ```
/// let mut girl = girl::Girl::new()?;
/// # if girl.gamepad(0).is_some() {
/// let mut gamepad = girl.gamepad(0).unwrap();
///
/// println!("{gamepad}");
/// // example output:
/// // PS4 Controller (Power: Wired)
/// # }
/// # Ok::<(), girl::Error>(())
/// ```
///
/// [`Gamepad`]: crate::Gamepad
impl fmt::Display for Gamepad {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        write!(f, "{}", if name.is_empty() { "Gamepad" } else { &name })?;
        if let Some(power) = self.power() {
            write!(f, " ({power})")?;
        }
        write!(f, ", connected as #{}", self.gp.instance_id())?;
        Ok(())
    }
}

impl Gamepad {
    /// Default deadzone value for analog sticks.
    pub const STICK_DEADZONE: f64 = 0.1;

    /// Creates a [`Gamepad`] from SDL controller and joystick handles.
    #[must_use]
    #[inline]
    #[cfg_attr(
        not(feature = "touchpad"),
        expect(
            clippy::missing_const_for_fn,
            clippy::unnecessary_wraps,
            reason = "feature gated"
        )
    )]
    pub(crate) fn from_sdl(
        controller: SdlController,
        joystick: SdlJoystick,
    ) -> Option<Self> {
        #[cfg_attr(
            not(feature = "touchpad"),
            expect(unused_mut, reason = "feature gated")
        )]
        let mut this = Self {
            joy: joystick,
            #[cfg(feature = "touchpad")]
            touchpads: vec![],
            gp: controller,
        };

        #[cfg(feature = "touchpad")]
        {
            this.touchpads = this.touchpads_init().ok()?;
        }

        Some(this)
    }

    /// Checks if the controller is currently connected.
    ///
    /// Disconnected [`Gamepad`]s will not report any input, but will still be
    /// available for use. When the controller reconnects, it will most likely
    /// have the same index as before, so you can replace the old [`Gamepad`]
    /// with the new call to [`Girl::gamepad`].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// // in a loop:
    /// if !gamepad.connected() {
    ///     // controller disconnected, reconnect it again once connected
    ///     if let Some(gp) = girl.gamepad(0) {
    ///         gamepad = gp;
    ///     }
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    ///
    /// [`Girl::gamepad`]: crate::Girl::gamepad
    #[must_use]
    #[inline]
    pub fn connected(&self) -> bool {
        self.gp.attached()
    }

    /// Gets the name of the [`Gamepad`] or an empty string if the name is not
    /// found.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// println!("{}", gamepad.name());
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn name(&self) -> String {
        self.gp.name()
    }

    /// Gets the current [`PowerLevel`] of the [`Gamepad`], if available.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::PowerLevel;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if let Some(power) = gamepad.power() {
    ///     println!("Power level: {power} [{}]", match power {
    ///         PowerLevel::Unknown => ":(",
    ///         PowerLevel::Empty => "dead X|",
    ///         PowerLevel::Low => "Charge your gamepad!",
    ///         PowerLevel::Medium | PowerLevel::Full => "You're good",
    ///         PowerLevel::Wired => "Nothing to worry about :)",
    ///     });
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn power(&self) -> Option<PowerLevel> {
        self.joy.power_level().ok().map(PowerLevel::from_sdl)
    }

    /// Query whether the [`Gamepad`] has an LED.
    #[must_use]
    #[inline]
    pub fn has_led(&self) -> bool {
        self.gp.has_led()
    }

    /// Sets the LED color on the [`Gamepad`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Gamepad`] doesn't have an LED or the operation
    /// fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_led() {
    ///     /// Set the LED to bright red
    ///     gamepad.set_led(255, 0, 0)?;
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[inline]
    pub fn set_led(
        &mut self,
        red: u8,
        green: u8,
        blue: u8,
    ) -> Result<(), Error> {
        self.gp
            .set_led(red, green, blue)
            .map_err(|err| Error::SdlError(err.to_string()))
    }
}

impl PartialEq for Gamepad {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.gp.instance_id() == other.gp.instance_id()
            && self.joy.instance_id() == other.joy.instance_id()
    }
}

impl Eq for Gamepad {}

impl PartialOrd for Gamepad {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Gamepad {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.gp.instance_id(), self.joy.instance_id())
            .cmp(&(other.gp.instance_id(), other.joy.instance_id()))
    }
}

impl hash::Hash for Gamepad {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.gp.instance_id().hash(state);
        self.joy.instance_id().hash(state);
    }
}

/// Battery power level of a [`Gamepad`].
#[expect(
    clippy::exhaustive_enums,
    reason = "if we get more power levels in the sdl2 updates, we'll add them \
              in a major update"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PowerLevel {
    /// Power level unknown.
    Unknown,

    /// Battery is empty.
    Empty,

    /// Battery level is low.
    Low,

    /// Battery level is medium.
    Medium,

    /// Battery level is full.
    Full,

    /// Device is wired (plugged in or has no battery).
    Wired,
}

impl fmt::Display for PowerLevel {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Power: ")?;
        match *self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Empty => write!(f, "Empty"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::Full => write!(f, "Full"),
            Self::Wired => write!(f, "Wired"),
        }
    }
}

impl PowerLevel {
    /// Converts from [`SdlPowerLevel`].
    #[must_use]
    #[inline]
    #[expect(clippy::single_call_fn, reason = "extracted conversion")]
    const fn from_sdl(level: SdlPowerLevel) -> Self {
        match level {
            SdlPowerLevel::Unknown => Self::Unknown,
            SdlPowerLevel::Empty => Self::Empty,
            SdlPowerLevel::Low => Self::Low,
            SdlPowerLevel::Medium => Self::Medium,
            SdlPowerLevel::Full => Self::Full,
            SdlPowerLevel::Wired => Self::Wired,
        }
    }
}

/// Maps a raw input value with deadzone and normalization.
pub(crate) fn map(value: f64, threshold: f64, max: f64) -> f64 {
    let value = value / max;
    if value.abs() < threshold { 0. } else { value }
}
