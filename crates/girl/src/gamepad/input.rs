//! [`Gamepad`] input types.

use sdl2::{
    controller::{Axis as SdlAxis, Button as SdlButton},
    sys::SDL_JOYSTICK_AXIS_MAX,
};

use crate::{Gamepad, gamepad::map};

/// Maximum value for analog axis inputs.
pub(crate) const AXIS_MAX: f64 = SDL_JOYSTICK_AXIS_MAX as f64;

/// [`Gamepad`] inputs.
// TODO: Try remove on next Rust version update.
#[expect(clippy::allow_attributes, reason = "`#[expect]` doesn't work here")]
#[allow(clippy::multiple_inherent_impl, reason = "documented implementation")]
impl Gamepad {
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
    pub(crate) const fn into_sdl_axis_pair(self) -> (SdlAxis, SdlAxis) {
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
    pub(crate) const fn into_sdl_axis(self) -> SdlAxis {
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
    pub(crate) fn into_sdl(self) -> SdlButton {
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
