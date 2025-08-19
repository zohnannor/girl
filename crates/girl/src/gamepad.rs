//! [`Gamepad`] and related types.

use core::{cmp, fmt, hash, time::Duration};

#[cfg(feature = "sensors")]
use sdl2::sensor::SensorType as SdlSensorType;
use sdl2::{
    controller::{
        Axis as SdlAxis, Button as SdlButton, GameController as SdlController,
    },
    event::Event as SdlEvent,
    joystick::{Joystick as SdlJoystick, PowerLevel as SdlPowerLevel},
    sys::{self as sdl2_sys, SDL_JOYSTICK_AXIS_MAX},
};

use crate::Error;

/// Maximum value for analog axis inputs.
pub(crate) const AXIS_MAX: f64 = SDL_JOYSTICK_AXIS_MAX as f64;

/// SDL2 released state constant.
#[expect(
    clippy::cast_possible_truncation,
    reason = "these constants should've been `Uint8` in the first place"
)]
const RELEASED: u8 = sdl2_sys::SDL_RELEASED as u8;

/// SDL2 pressed state constant.
#[expect(
    clippy::cast_possible_truncation,
    reason = "these constants should've been `Uint8` in the first place"
)]
const PRESSED: u8 = sdl2_sys::SDL_PRESSED as u8;

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
/// # Ok::<(), girl::Error>(())
///
/// // check buttons, sensors, etc.
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
    pub(crate) fn from_sdl(
        controller: SdlController,
        joystick: SdlJoystick,
    ) -> Option<Self> {
        let mut this =
            Self { joy: joystick, touchpads: vec![], gp: controller };

        this.touchpads = this.touchpads_init().ok()?;

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

    /// Gets the current position of an analog [`Stick`] with default
    /// [`STICK_DEADZONE`] threshold.
    ///
    /// Values are in the range `[-1.0, 1.0]`, where `x` is from left to right
    /// and `y` is from **top** to **bottom**.
    ///
    /// ```text
    ///           -1.0
    ///       +-----^-----+
    ///       |     |     |
    ///       |     |     |
    ///   -1.0<-----+----->+1.0
    ///       |     |     |
    ///       |     |     |
    ///       +-----v-----+
    ///           +1.0
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Stick;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// let [x, y] = gamepad.stick(Stick::Right);
    /// // apply movement to a character, etc.
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    ///
    /// [`STICK_DEADZONE`]: Self::STICK_DEADZONE
    #[must_use]
    #[inline]
    pub fn stick(&self, stick: Stick) -> [f64; 2] {
        self.stick_with_deadzone(stick, Self::STICK_DEADZONE)
    }

    /// Gets the current position of an analog [`Stick`] with the provided
    /// `deadzone` threshold.
    ///
    /// Values are in the range `[-1.0, 1.0]`, where `x` is from left to right
    /// and `y` is from **top** to **bottom**.
    ///
    /// ```text
    ///           -1.0
    ///       +-----^-----+
    ///       |     |     |
    ///       |     |     |
    ///   -1.0<-----+----->+1.0
    ///       |     |     |
    ///       |     |     |
    ///       +-----v-----+
    ///           +1.0
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Stick;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// let [x, y] = gamepad.stick_with_deadzone(Stick::Right, 0.05);
    /// // apply movement to a character, etc.
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    ///
    /// [`STICK_DEADZONE`]: Self::STICK_DEADZONE
    #[must_use]
    #[inline]
    pub fn stick_with_deadzone(&self, stick: Stick, deadzone: f64) -> [f64; 2] {
        let (x, y) = stick.into_sdl_axis_pair();
        [
            map(self.gp.axis(x).into(), deadzone, AXIS_MAX),
            map(self.gp.axis(y).into(), deadzone, AXIS_MAX),
        ]
    }

    /// Gets the current value of a [`Trigger`].
    ///
    /// Value is in the range `[-1.0, 1.0]`, where `0.0` is the rest position
    /// and `1.0` is fully pressed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Trigger;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// let right_trigger = gamepad.trigger(Trigger::Right);
    /// // apply movement to a character, etc.
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn trigger(&self, trigger: Trigger) -> f64 {
        map(self.gp.axis(trigger.into_sdl_axis()).into(), 0.0, AXIS_MAX)
    }

    /// Gets the current state of the specified [`Button`]\(s).
    ///
    /// Allows to query multiple [`Button`]\(s) at once.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Button;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// let buttons = gamepad.buttons(Button::A | Button::B);
    /// // check if both buttons are pressed
    /// if buttons.contains(Button::A) && buttons.contains(Button::B) {}
    /// // or like this:
    /// if buttons.contains(Button::A | Button::B) {}
    /// // or like this (only those two buttons are pressed):
    /// if buttons == Button::A | Button::B {}
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn buttons(&self, buttons: Button) -> Button {
        buttons
            .iter()
            .filter(|button: &Button| self.gp.button(button.into_sdl()))
            .collect()
    }

    /// Checks if all specified [`Button`]\(s) are currently pressed.
    ///
    /// Allows to query multiple [`Button`]\(s) at once.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Button;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// // check if both buttons are pressed
    /// if gamepad.buttons_pressed(Button::A | Button::B) {}
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn buttons_pressed(&self, buttons: Button) -> bool {
        self.buttons(buttons) == buttons
    }

    /// Query whether the [`Gamepad`] has touchpads.
    #[must_use]
    #[inline]
    pub const fn has_touchpads(&self) -> usize {
        self.touchpads.len()
    }

    /// Gets the current [`TouchpadState`]\(s).
    ///
    /// Returns a [`Vec`] of [`TouchpadState`]\(s) (for every finger that
    /// touched any of available touchpads) if any are available.
    ///
    /// - [`TouchpadAction::Released`] means the finger was just released from
    ///   the touchpad, and is returned **once** when the finger is released.
    /// - [`TouchpadAction::Touched`] means the finger was just touched on the
    ///   touchpad, and is returned **once** the finger is touched.
    /// - [`TouchpadAction::Moved`] means the finger has been touching the
    ///   touchpad and has moved, and is returned **every time** the finger is
    ///   moved and the postiion is updated.
    ///
    /// If no touchpads are touched, returns an empty [`Vec`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the [`Gamepad`] is no longer valid.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_touchpads() > 0 {
    ///     let touchpads = gamepad.touchpad()?;
    ///     for touchpad in touchpads {
    ///         // do something with touchpad state values
    ///         let [x, y] = touchpad.position;
    ///         match touchpad.action {
    ///             girl::TouchpadAction::Released => {}
    ///             girl::TouchpadAction::Touched => {}
    ///             girl::TouchpadAction::Moved => {}
    ///         }
    ///     }
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[inline]
    pub fn touchpad(&mut self) -> Result<Vec<TouchpadState>, Error> {
        let raw = self.raw()?;

        let mut states = vec![];

        for (touchpad_idx, touchpad) in self.touchpads.iter_mut().enumerate() {
            for (finger_idx, prev) in touchpad.iter_mut().enumerate() {
                use self::TouchpadAction as TA;

                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_possible_wrap,
                    reason = "ok to cast"
                )]
                let (finger, idx) = (finger_idx as i32, touchpad_idx as i32);

                let mut position = [0.0, 0.0];
                let mut pressure = 0.0;
                let mut state = 0;

                // SAFETY: SDL2 is still alive, all the pointers are valid.
                #[expect(unsafe_code, reason = "ffi with sdl2")]
                let res = unsafe {
                    sdl2_sys::SDL_GameControllerGetTouchpadFinger(
                        raw,
                        idx,
                        finger,
                        &raw mut state,
                        &raw mut position[0],
                        &raw mut position[1],
                        &raw mut pressure,
                    )
                };

                if res != 0i32 {
                    continue;
                }

                let action = match state {
                    RELEASED => TA::Released,
                    PRESSED => TA::Touched,
                    _ => unreachable!("unknown touchpad state: {state}"),
                };

                let event_type = if action == prev.action {
                    // only report the first release event
                    if action == TA::Released {
                        continue;
                    }

                    // don't report the same event twice
                    #[expect(
                        clippy::float_cmp,
                        reason = "want this to be the same as the sdl2 logic"
                    )]
                    if position == prev.position && pressure == prev.pressure {
                        continue;
                    }

                    // otherwise, report the repeated touch as a move event
                    TA::Moved
                } else if action == TA::Touched {
                    TA::Touched
                } else {
                    TA::Released
                };

                prev.action = action;
                prev.position = position;
                prev.pressure = pressure;

                states.push(TouchpadState {
                    touchpad: touchpad_idx,
                    finger: finger_idx,
                    position,
                    pressure,
                    action: event_type,
                });
            }
        }

        Ok(states)
    }

    /// Gets the raw SDL game controller pointer.
    ///
    /// # Errors
    ///
    /// Returns an error if the controller is no longer valid.
    #[inline]
    fn raw(&self) -> Result<*mut sdl2_sys::SDL_GameController, Error> {
        #[expect(
            clippy::cast_possible_wrap,
            reason = "it was just cast from i32 to u32 by sdl2 crate, we're \
                      casting it back"
        )]
        let id = self.gp.instance_id() as i32;

        // SAFETY: SDL is alive, `id` is valid, and SDL handles any errors,
        //         return value is checked for null.
        #[expect(unsafe_code, reason = "ffi with sdl2")]
        let res = unsafe { sdl2_sys::SDL_GameControllerFromInstanceID(id) };

        if res.is_null() {
            Err(Error::SdlError(sdl2::get_error()))
        } else {
            Ok(res)
        }
    }

    /// Creates touchpad state storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the controller is no longer valid.
    #[inline]
    fn touchpads_init(&self) -> Result<Vec<Vec<TouchpadState>>, Error> {
        let raw = self.raw()?;

        // SAFETY: SDL is alive, pointer is valid
        #[expect(unsafe_code, reason = "ffi with sdl2")]
        let num_touchpads =
            unsafe { sdl2_sys::SDL_GameControllerGetNumTouchpads(raw) };

        #[expect(
            clippy::cast_sign_loss,
            reason = "ok to cast after checking for error"
        )]
        let touchpads =
            if num_touchpads < 0i32 { 0 } else { num_touchpads as usize };

        // SAFETY: SDL is alive, pointer is valid
        #[expect(unsafe_code, reason = "ffi with sdl2")]
        let fingers = unsafe {
            sdl2_sys::SDL_GameControllerGetNumTouchpadFingers(raw, 0)
        };

        #[expect(
            clippy::cast_sign_loss,
            reason = "ok to cast after checking for error"
        )]
        let fingers = if fingers < 0i32 { 0 } else { fingers as usize };

        Ok(vec![vec![TouchpadState::default(); fingers]; touchpads])
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

    /// Query whether the [`Gamepad`] has rumble support.
    #[must_use]
    #[inline]
    pub fn has_rumble(&self) -> bool {
        self.gp.has_rumble()
    }

    /// Sets the rumble intensity and duration. Automatically resets back to
    /// zero after `duration` has passed.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Gamepad`] doesn't support rumble or the
    /// operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time::Duration;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_rumble() {
    ///     gamepad.set_rumble(1000, 1, Duration::from_millis(100))?;
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[inline]
    pub fn set_rumble(
        &mut self,
        low_frequency_rumble: u16,
        high_frequency_rumble: u16,
        duration: Duration,
    ) -> Result<(), Error> {
        self.gp
            .set_rumble(
                low_frequency_rumble,
                high_frequency_rumble,
                duration.as_millis().try_into().unwrap_or(u32::MAX),
            )
            .map_err(|err| Error::SdlError(err.to_string()))
    }

    /// Stops rumble effects.
    ///
    /// Analogous to [`set_rumble`] with `low_frequency_rumble` and
    /// `high_frequency_rumble` set to zero.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Gamepad`] doesn't support rumble or the
    /// operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_rumble() {
    ///     // set rumble before, then:
    ///     gamepad.end_rumble()?;
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    ///
    /// [`set_rumble`]: Self::set_rumble
    #[inline]
    pub fn end_rumble(&mut self) -> Result<(), Error> {
        self.set_rumble(0, 0, Duration::from_millis(1))
    }

    /// Query whether the gamepad has trigger rumble support.
    #[must_use]
    #[inline]
    pub fn has_rumble_triggers(&self) -> bool {
        self.gp.has_rumble_triggers()
    }

    /// Sets rumble intensity for the triggers.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Gamepad`] doesn't support trigger rumble or
    /// the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time::Duration;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_rumble_triggers() {
    ///     gamepad.set_rumble_triggers(1000, 1, Duration::from_millis(100))?;
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[inline]
    pub fn set_rumble_triggers(
        &mut self,
        left_trigger_rumble: u16,
        right_trigger_rumble: u16,
        duration: Duration,
    ) -> Result<(), Error> {
        self.gp
            .set_rumble_triggers(
                left_trigger_rumble,
                right_trigger_rumble,
                duration.as_millis().try_into().unwrap_or(u32::MAX),
            )
            .map_err(|err| Error::SdlError(err.to_string()))
    }

    /// Stops trigger rumble effects.
    ///
    /// Analogous to [`set_rumble_triggers`] with `left_trigger_rumble` and
    /// `right_trigger_rumble` set to zero.
    ///
    /// # Errors
    ///
    /// Returns an error if the [`Gamepad`] doesn't support trigger rumble or
    /// the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_rumble_triggers() {
    ///     // set rumble before, then:
    ///     gamepad.end_rumble_triggers()?;
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    ///
    /// [`set_rumble_triggers`]: Self::set_rumble_triggers
    #[inline]
    pub fn end_rumble_triggers(&mut self) -> Result<(), Error> {
        self.set_rumble_triggers(0, 0, Duration::from_millis(1))
    }
}

/// test
#[cfg(feature = "sensors")]
#[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
// TODO: Try remove on next Rust version update.
#[expect(clippy::allow_attributes, reason = "`#[expect]` doesn't work here")]
#[allow(
    clippy::multiple_inherent_impl,
    reason = "feature gated and documented"
)]
impl Gamepad {
    /// Query whether the gamepad has a specific sensor.
    #[must_use]
    #[inline]
    pub fn has_sensor(&self, sensor_type: Sensor) -> bool {
        self.gp.has_sensor(sensor_type.into_sdl())
    }

    /// Enables a [`Sensor`] on the [`Gamepad`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the sensor is not available or fails to enable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Sensor;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_sensor(Sensor::Gyroscope) {
    ///     gamepad.enable_sensor(Sensor::Gyroscope)?;
    ///     // read sensor data later
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[inline]
    pub fn enable_sensor(&self, sensor: Sensor) -> Result<(), Error> {
        self.gp
            .sensor_set_enabled(sensor.into_sdl(), true)
            .map_err(|err| Error::SdlError(err.to_string()))
    }

    /// Gets current [`Sensor`] data.
    ///
    /// You will need to enable the [`Sensor`] first using [`enable_sensor`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the [`Sensor`] is not available or fails to
    /// read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Sensor;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_sensor(Sensor::Gyroscope) {
    ///     gamepad.enable_sensor(Sensor::Gyroscope)?;
    ///     let [x, y, z] = gamepad.sensor(Sensor::Gyroscope)?;
    ///     // apply movement to a character, etc.
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    ///
    /// [`enable_sensor`]: Self::enable_sensor
    #[inline]
    pub fn sensor(&self, sensor: Sensor) -> Result<[f64; 3], Error> {
        let mut data = [0.; 3];
        self.gp
            .sensor_get_data(sensor.into_sdl(), &mut data)
            .map_err(|err| Error::SdlError(err.to_string()))?;
        Ok(data.map(|x| map(f64::from(x), 0.01, 1.)))
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

/// Analog sticks on a [`Gamepad`].
#[expect(
    clippy::exhaustive_enums,
    reason = "if gamepads get more sticks in the future, we'll add them in a \
              major update"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stick {
    /// Left analog stick.
    Left,
    /// Right analog stick.
    Right,
}

impl Stick {
    /// Converts to [`SdlAxis`] pair.
    #[must_use]
    #[inline]
    const fn into_sdl_axis_pair(self) -> (SdlAxis, SdlAxis) {
        match self {
            Self::Left => (SdlAxis::LeftX, SdlAxis::LeftY),
            Self::Right => (SdlAxis::RightX, SdlAxis::RightY),
        }
    }
}

/// Triggers on a [`Gamepad`].
#[expect(
    clippy::exhaustive_enums,
    reason = "if gamepads get more triggers in the future, we'll add them in a \
              major update"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Trigger {
    /// Left trigger.
    Left,
    /// Right trigger.
    Right,
}

impl Trigger {
    /// Converts to [`SdlAxis`].
    #[must_use]
    #[inline]
    const fn into_sdl_axis(self) -> SdlAxis {
        match self {
            Self::Left => SdlAxis::TriggerLeft,
            Self::Right => SdlAxis::TriggerRight,
        }
    }
}

bitflags::bitflags! {
    /// Gamepad buttons.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Button: u32 {
        /// A button (typically bottom button on the right side).
        ///
        /// "A" on the Xbox controller, "╳" on the PlayStation controller, "B"
        /// on the Nintendo Switch controller.
        const A = 1 << 0;

        /// B button (typically right button on the right side).
        ///
        /// "B" on the Xbox controller, "◯" on the PlayStation controller, "A"
        /// on the Nintendo Switch controller.
        const B = 1 << 1;

        /// X button (typically left button on the right side).
        ///
        /// "X" on the Xbox controller, "□" on the PlayStation controller, "Y"
        /// on the Nintendo Switch controller.
        const X = 1 << 2;

        /// Y button (typically top button on the right side).
        ///
        /// "Y" on the Xbox controller, "△" on the PlayStation controller, "X"
        /// on the Nintendo Switch controller.
        const Y = 1 << 3;

        /// Back/Select button.
        ///
        /// "Back" on the Xbox controller, "Share" on the PlayStation
        /// controller, "-" on the Nintendo Switch controller.
        const Back = 1 << 4;

        /// Guide/Home button.
        ///
        /// "<Xbox logo>" on the Xbox controller, "<PlayStation logo>" on the
        /// PlayStation controller, "🏠" on the Nintendo Switch controller.
        const Guide = 1 << 5;

        /// Start button.
        ///
        /// "Start" on the Xbox controller, "Options" on the PlayStation
        /// controller, "-" on the Nintendo Switch controller.
        const Start = 1 << 6;

        /// Left stick click button.
        ///
        /// "Left Stick Click" on the Xbox controller, "L3" on the PlayStation
        /// controller, "Left Stick Click" on the Nintendo Switch controller.
        const LeftStick = 1 << 7;

        /// Right stick click button.
        ///
        /// "Right Stick Click" on the Xbox controller, "R3" on the PlayStation
        /// controller, "Right Stick Click" on the Nintendo Switch controller.
        const RightStick = 1 << 8;

        /// Left shoulder button.
        ///
        /// "Left Bumper (LB)" on the Xbox controller, "L1" on the PlayStation
        /// controller, "L" on the Nintendo Switch controller.
        const LeftShoulder = 1 << 9;

        /// Right shoulder button.
        ///
        /// "Right Bumper (RB)" on the Xbox controller, "R1" on the PlayStation
        /// controller, "R" on the Nintendo Switch controller.
        const RightShoulder = 1 << 10;

        /// D-pad up button.
        const DPadUp = 1 << 11;

        /// D-pad down button.
        const DPadDown = 1 << 12;

        /// D-pad left button.
        const DPadLeft = 1 << 13;

        /// D-pad right button.
        const DPadRight = 1 << 14;

        /// Miscellaneous button 1.
        ///
        /// "Share" on the Xbox Series X/S controller, "Microphone" on the
        /// PlayStation 5 controller, "Capture button" on the Nintendo Switch
        /// controller.
        const Misc1 = 1 << 15;

        /// Paddle 1.
        ///
        /// "Paddle 1" on Xbox Elite controllers (upper left, facing the back),
        /// not available on standard PlayStation or Nintendo Switch
        /// controllers.
        const Paddle1 = 1 << 16;

        /// Paddle 2.
        ///
        /// "Paddle 2" on Xbox Elite controllers (upper right, facing the back),
        /// not available on standard PlayStation or Nintendo Switch
        /// controllers.
        const Paddle2 = 1 << 17;

        /// Paddle 3.
        ///
        /// "Paddle 3" on Xbox Elite controllers (lower left, facing the back),
        /// not available on standard PlayStation or Nintendo Switch
        /// controllers.
        const Paddle3 = 1 << 18;

        /// Paddle 4.
        ///
        /// "Paddle 4" on Xbox Elite controllers (lower right, facing the back),
        /// not available on standard PlayStation or Nintendo Switch
        /// controllers.
        const Paddle4 = 1 << 19;

        /// Touchpad button.
        ///
        /// Not available on standard Xbox controllers, "Touchpad button" on the
        /// PlayStation 4/5 controller (pressing on a touchpad), not available
        /// on standard Nintendo Switch controllers.
        const Touchpad = 1 << 20;
    }
}

impl Button {
    /// Converts from SDL button.
    #[must_use]
    #[inline]
    pub(crate) const fn from_sdl(button: SdlButton) -> Self {
        match button {
            SdlButton::A => Self::A,
            SdlButton::B => Self::B,
            SdlButton::X => Self::X,
            SdlButton::Y => Self::Y,
            SdlButton::Back => Self::Back,
            SdlButton::Guide => Self::Guide,
            SdlButton::Start => Self::Start,
            SdlButton::LeftStick => Self::LeftStick,
            SdlButton::RightStick => Self::RightStick,
            SdlButton::LeftShoulder => Self::LeftShoulder,
            SdlButton::RightShoulder => Self::RightShoulder,
            SdlButton::DPadUp => Self::DPadUp,
            SdlButton::DPadDown => Self::DPadDown,
            SdlButton::DPadLeft => Self::DPadLeft,
            SdlButton::DPadRight => Self::DPadRight,
            SdlButton::Misc1 => Self::Misc1,
            SdlButton::Paddle1 => Self::Paddle1,
            SdlButton::Paddle2 => Self::Paddle2,
            SdlButton::Paddle3 => Self::Paddle3,
            SdlButton::Paddle4 => Self::Paddle4,
            SdlButton::Touchpad => Self::Touchpad,
        }
    }

    /// Converts to SDL button.
    #[must_use]
    #[inline]
    fn into_sdl(self) -> SdlButton {
        bitflags::bitflags_match!(self, {
            Self::A => SdlButton::A,
            Self::B => SdlButton::B,
            Self::X => SdlButton::X,
            Self::Y => SdlButton::Y,
            Self::Back => SdlButton::Back,
            Self::Guide => SdlButton::Guide,
            Self::Start => SdlButton::Start,
            Self::LeftStick => SdlButton::LeftStick,
            Self::RightStick => SdlButton::RightStick,
            Self::LeftShoulder => SdlButton::LeftShoulder,
            Self::RightShoulder => SdlButton::RightShoulder,
            Self::DPadUp => SdlButton::DPadUp,
            Self::DPadDown => SdlButton::DPadDown,
            Self::DPadLeft => SdlButton::DPadLeft,
            Self::DPadRight => SdlButton::DPadRight,
            Self::Misc1 => SdlButton::Misc1,
            Self::Paddle1 => SdlButton::Paddle1,
            Self::Paddle2 => SdlButton::Paddle2,
            Self::Paddle3 => SdlButton::Paddle3,
            Self::Paddle4 => SdlButton::Paddle4,
            Self::Touchpad => SdlButton::Touchpad,
            _ => unreachable!("use only with single button bit set"),
        })
    }
}

/// Sensors available on gamepads.
#[cfg(feature = "sensors")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "if gamepads get more sensors in the future, we'll add them in a \
              major update"
)]
pub enum Sensor {
    /// Unknown sensor type.
    Unknown,

    /// Gyroscope.
    Gyroscope,

    /// Gyroscope for left Joy-Con controller .
    LeftGyroscope,

    /// Gyroscope for right Joy-Con controller.
    RightGyroscope,

    /// Accelerometer.
    Accelerometer,

    /// Accelerometer for left Joy-Con controller.
    LeftAccelerometer,

    /// Accelerometer for right Joy-Con controller.
    RightAccelerometer,
}

#[cfg(feature = "sensors")]
#[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
impl Sensor {
    /// Converts from [`SdlSensorType`].
    #[must_use]
    #[inline]
    #[expect(clippy::single_call_fn, reason = "extracted conversion")]
    pub(crate) const fn from_sdl(sensor: SdlSensorType) -> Self {
        match sensor {
            SdlSensorType::Unknown => Self::Unknown,
            SdlSensorType::Gyroscope => Self::Gyroscope,
            SdlSensorType::LeftGyroscope => Self::LeftGyroscope,
            SdlSensorType::RightGyroscope => Self::RightGyroscope,
            SdlSensorType::Accelerometer => Self::Accelerometer,
            SdlSensorType::LeftAccelerometer => Self::LeftAccelerometer,
            SdlSensorType::RightAccelerometer => Self::RightAccelerometer,
        }
    }

    /// Converts to [`SdlSensorType`].
    #[must_use]
    #[inline]
    const fn into_sdl(self) -> SdlSensorType {
        match self {
            Self::Unknown => SdlSensorType::Unknown,
            Self::Gyroscope => SdlSensorType::Gyroscope,
            Self::LeftGyroscope => SdlSensorType::LeftGyroscope,
            Self::RightGyroscope => SdlSensorType::RightGyroscope,
            Self::Accelerometer => SdlSensorType::Accelerometer,
            Self::LeftAccelerometer => SdlSensorType::LeftAccelerometer,
            Self::RightAccelerometer => SdlSensorType::RightAccelerometer,
        }
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

/// Touchpad event with position, pressure, and action.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct TouchpadEvent {
    /// Controller instance ID.
    pub which: u32,
    /// Touchpad index.
    pub idx: u32,
    /// Finger index.
    pub finger: u32,
    /// Normalized position `[x, y]` where both values range from 0.0 to 1.0.
    pub position: [f32; 2],
    /// Normalized pressure from 0.0 to 1.0.
    pub pressure: f32,
    /// Type of touch action.
    pub action: TouchpadAction,
}

/// Type of touchpad action
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[expect(clippy::exhaustive_enums, reason = "no more actions possible")]
pub enum TouchpadAction {
    /// Finger touched the touchpad.
    Touched,
    /// Finger released from the touchpad.
    #[default]
    Released,
    /// Finger moved on the touchpad.
    Moved,
}

impl TouchpadEvent {
    /// Converts from SDL event.
    #[must_use]
    #[inline]
    #[expect(clippy::too_many_lines, reason = "not much we can do")]
    pub const fn from_sdl(event: &SdlEvent) -> Option<Self> {
        Some(match *event {
            SdlEvent::ControllerTouchpadDown {
                which,
                touchpad,
                finger,
                x,
                y,
                pressure,
                ..
            } => Self {
                which,
                idx: touchpad,
                finger,
                position: [x, y],
                pressure,
                action: TouchpadAction::Touched,
            },
            SdlEvent::ControllerTouchpadUp {
                which,
                touchpad,
                finger,
                x,
                y,
                pressure,
                ..
            } => Self {
                which,
                idx: touchpad,
                finger,
                position: [x, y],
                pressure,
                action: TouchpadAction::Released,
            },
            SdlEvent::ControllerTouchpadMotion {
                which,
                touchpad,
                finger,
                x,
                y,
                pressure,
                ..
            } => Self {
                which,
                idx: touchpad,
                finger,
                position: [x, y],
                pressure,
                action: TouchpadAction::Moved,
            },
            SdlEvent::Quit { .. }
            | SdlEvent::AppTerminating { .. }
            | SdlEvent::AppLowMemory { .. }
            | SdlEvent::AppWillEnterBackground { .. }
            | SdlEvent::AppDidEnterBackground { .. }
            | SdlEvent::AppWillEnterForeground { .. }
            | SdlEvent::AppDidEnterForeground { .. }
            | SdlEvent::Display { .. }
            | SdlEvent::Window { .. }
            | SdlEvent::KeyDown { .. }
            | SdlEvent::KeyUp { .. }
            | SdlEvent::TextEditing { .. }
            | SdlEvent::TextInput { .. }
            | SdlEvent::MouseMotion { .. }
            | SdlEvent::MouseButtonDown { .. }
            | SdlEvent::MouseButtonUp { .. }
            | SdlEvent::MouseWheel { .. }
            | SdlEvent::JoyAxisMotion { .. }
            | SdlEvent::JoyBallMotion { .. }
            | SdlEvent::JoyHatMotion { .. }
            | SdlEvent::JoyButtonDown { .. }
            | SdlEvent::JoyButtonUp { .. }
            | SdlEvent::JoyDeviceAdded { .. }
            | SdlEvent::JoyDeviceRemoved { .. }
            | SdlEvent::ControllerAxisMotion { .. }
            | SdlEvent::ControllerButtonDown { .. }
            | SdlEvent::ControllerButtonUp { .. }
            | SdlEvent::ControllerDeviceAdded { .. }
            | SdlEvent::ControllerDeviceRemoved { .. }
            | SdlEvent::ControllerDeviceRemapped { .. }
            | SdlEvent::ControllerSteamHandleUpdate { .. }
            | SdlEvent::FingerDown { .. }
            | SdlEvent::FingerUp { .. }
            | SdlEvent::FingerMotion { .. }
            | SdlEvent::DollarGesture { .. }
            | SdlEvent::DollarRecord { .. }
            | SdlEvent::MultiGesture { .. }
            | SdlEvent::ClipboardUpdate { .. }
            | SdlEvent::DropFile { .. }
            | SdlEvent::DropText { .. }
            | SdlEvent::DropBegin { .. }
            | SdlEvent::DropComplete { .. }
            | SdlEvent::AudioDeviceAdded { .. }
            | SdlEvent::AudioDeviceRemoved { .. }
            | SdlEvent::RenderTargetsReset { .. }
            | SdlEvent::RenderDeviceReset { .. }
            | SdlEvent::LocaleChanged { .. }
            | SdlEvent::User { .. }
            | SdlEvent::Unknown { .. } => return None,
            #[cfg(feature = "sensors")]
            SdlEvent::ControllerSensorUpdated { .. } => return None,
        })
    }
}

/// Touchpad state for each touchpad and finger.
///
/// Returned by [`Gamepad::touchpad`].
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
#[non_exhaustive]
pub struct TouchpadState {
    /// Touchpad index.
    pub touchpad: usize,
    /// Finger index.
    pub finger: usize,
    /// Normalized position [x, y] where both values range from 0.0 to 1.0.
    pub position: [f32; 2],
    /// Normalized pressure from 0.0 to 1.0.
    pub pressure: f32,
    /// Type of touch action.
    pub action: TouchpadAction,
}

/// Maps a raw input value with deadzone and normalization
pub(crate) fn map(value: f64, threshold: f64, max: f64) -> f64 {
    let value = value / max;
    if value.abs() < threshold { 0. } else { value }
}
