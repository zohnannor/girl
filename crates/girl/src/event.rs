//! Input event types and conversion from SDL events.

use sdl2::{controller::Axis as SdlAxis, event::Event as SdlEvent};

#[cfg(feature = "sensors")]
use crate::Sensor;
#[cfg(feature = "touchpad")]
use crate::TouchpadEvent;
use crate::{
    Button, Gamepad, Stick, Trigger,
    gamepad::{input::AXIS_MAX, map},
};

/// Input events that can be processed by the library.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// Application quit requested.
    Quit,

    /// Analog stick movement.
    ControllerStickMotion {
        /// Controller instance ID.
        which: u32,
        /// Which stick moved.
        stick: Stick,
        /// Raw stick values `[x, y]`.
        offset: [f64; 2],
    },

    /// Trigger movement.
    ControllerTriggerMotion {
        /// Controller instance ID.
        which: u32,
        /// Which trigger moved.
        trigger: Trigger,
        /// Raw trigger value.
        offset: f64,
    },

    /// Button pressed.
    ControllerButtonDown {
        /// Controller instance ID.
        which: u32,
        /// Button that was pressed.
        button: Button,
    },

    /// Button released.
    ControllerButtonUp {
        /// Controller instance ID.
        which: u32,
        /// Button that was released.
        button: Button,
    },

    /// New controller connected.
    ControllerDeviceAdded {
        /// Controller instance ID.
        which: u32,
    },

    /// Controller disconnected.
    ControllerDeviceRemoved {
        /// Controller instance ID.
        which: u32,
    },

    /// Controller button mapping changed.
    ControllerDeviceRemapped {
        /// Controller instance ID.
        which: u32,
    },

    /// Steam controller handle updated.
    ControllerSteamHandleUpdate {
        /// Controller instance ID.
        which: u32,
    },

    /// Touchpad event.
    #[cfg(feature = "touchpad")]
    #[cfg_attr(docsrs, doc(cfg(feature = "touchpad")))]
    ControllerTouchpad(TouchpadEvent),

    /// Sensor data updated.
    #[cfg(feature = "sensors")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
    ControllerSensorUpdated {
        /// Controller instance ID.
        which: u32,
        /// Type of sensor.
        sensor: Sensor,
        /// Sensor data `[x, y, z]`.
        data: [f64; 3],
    },
}

impl Event {
    /// Converts from [`SdlEvent`] to [`Event`].
    #[expect(clippy::too_many_lines, reason = "not much we can do")]
    #[must_use]
    #[inline]
    pub(crate) fn from_sdl(event: &SdlEvent) -> Option<Self> {
        Some(match *event {
            SdlEvent::Quit { timestamp: _ } => Self::Quit,
            SdlEvent::ControllerAxisMotion {
                timestamp: _,
                which,
                axis: axis @ (SdlAxis::LeftX | SdlAxis::LeftY),
                value,
            } => Self::ControllerStickMotion {
                which,
                stick: Stick::Left,
                offset: if axis == SdlAxis::LeftX {
                    [0.0, map(value.into(), Gamepad::STICK_DEADZONE, AXIS_MAX)]
                } else {
                    [map(value.into(), Gamepad::STICK_DEADZONE, AXIS_MAX), 0.0]
                },
            },
            SdlEvent::ControllerAxisMotion {
                timestamp: _,
                which,
                axis: axis @ (SdlAxis::RightX | SdlAxis::RightY),
                value,
            } => Self::ControllerStickMotion {
                which,
                stick: Stick::Right,
                offset: if axis == SdlAxis::LeftX {
                    [map(value.into(), Gamepad::STICK_DEADZONE, AXIS_MAX), 0.0]
                } else {
                    [0.0, map(value.into(), Gamepad::STICK_DEADZONE, AXIS_MAX)]
                },
            },
            SdlEvent::ControllerAxisMotion {
                timestamp: _,
                which,
                axis: SdlAxis::TriggerLeft,
                value,
            } => Self::ControllerTriggerMotion {
                which,
                trigger: Trigger::Left,
                offset: map(value.into(), 0.0, AXIS_MAX),
            },
            SdlEvent::ControllerAxisMotion {
                timestamp: _,
                which,
                axis: SdlAxis::TriggerRight,
                value,
            } => Self::ControllerTriggerMotion {
                which,
                trigger: Trigger::Right,
                offset: map(value.into(), 0.0, AXIS_MAX),
            },
            SdlEvent::ControllerButtonDown { timestamp: _, which, button } => {
                Self::ControllerButtonDown {
                    which,
                    button: Button::from_sdl(button),
                }
            }
            SdlEvent::ControllerButtonUp { timestamp: _, which, button } => {
                Self::ControllerButtonUp {
                    which,
                    button: Button::from_sdl(button),
                }
            }
            SdlEvent::ControllerDeviceAdded { timestamp: _, which } => {
                Self::ControllerDeviceAdded { which }
            }
            SdlEvent::ControllerDeviceRemoved { timestamp: _, which } => {
                Self::ControllerDeviceRemoved { which }
            }
            SdlEvent::ControllerDeviceRemapped { timestamp: _, which } => {
                Self::ControllerDeviceRemapped { which }
            }
            SdlEvent::ControllerSteamHandleUpdate { timestamp: _, which } => {
                Self::ControllerSteamHandleUpdate { which }
            }
            #[cfg(feature = "touchpad")]
            SdlEvent::ControllerTouchpadDown { .. } => {
                Self::ControllerTouchpad(TouchpadEvent::from_sdl(event)?)
            }
            #[cfg(feature = "touchpad")]
            SdlEvent::ControllerTouchpadMotion { .. } => {
                Self::ControllerTouchpad(TouchpadEvent::from_sdl(event)?)
            }
            #[cfg(feature = "touchpad")]
            SdlEvent::ControllerTouchpadUp { .. } => {
                Self::ControllerTouchpad(TouchpadEvent::from_sdl(event)?)
            }
            #[cfg(not(feature = "touchpad"))]
            SdlEvent::ControllerTouchpadDown { .. }
            | SdlEvent::ControllerTouchpadMotion { .. }
            | SdlEvent::ControllerTouchpadUp { .. } => return None,
            #[cfg(feature = "sensors")]
            SdlEvent::ControllerSensorUpdated {
                timestamp: _,
                which,
                sensor,
                data,
            } => Self::ControllerSensorUpdated {
                which,
                sensor: Sensor::from_sdl(sensor),
                data: data.map(|x| map(f64::from(x), 0.01, 1.)),
            },
            SdlEvent::AppTerminating { .. }
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
        })
    }
}
