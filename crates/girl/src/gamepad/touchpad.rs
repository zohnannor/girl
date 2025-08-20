//! Touchpad data for a [`Gamepad`].

use alloc::{vec, vec::Vec};

use sdl2::{event::Event as SdlEvent, sys as sdl2_sys};

use crate::{Error, Gamepad};

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

/// Touchpad data for a [`Gamepad`].
#[cfg_attr(docsrs, doc(cfg(feature = "touchpad")))]
// TODO: Try remove on next Rust version update.
#[expect(clippy::allow_attributes, reason = "`#[expect]` doesn't work here")]
#[allow(
    clippy::multiple_inherent_impl,
    reason = "feature gated and documented"
)]
impl Gamepad {
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
    pub(crate) fn touchpads_init(
        &self,
    ) -> Result<Vec<Vec<TouchpadState>>, Error> {
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

/// Type of touchpad action.
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
