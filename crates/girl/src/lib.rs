#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! <br>
#![cfg_attr(docsrs, feature(doc_cfg))]

mod event;
mod gamepad;
mod gamepadmanager;

#[cfg(test)]
mod unused {
    use tracing_subscriber as _;
}

use tracing as _;

#[cfg(feature = "sensors")]
#[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
pub use crate::gamepad::Sensor;
pub use crate::{
    event::Event,
    gamepad::{
        Button, Gamepad, PowerLevel, Stick, TouchpadAction, TouchpadEvent,
        TouchpadState, Trigger,
    },
    gamepadmanager::{ConnectedGamepads, Girl},
};

/// Error types that can occur when working with gamepad input.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// SDL2 failed to initialize.
    Sdl2Init(String),

    /// An error occurred in the SDL2 subsystem.
    SdlError(String),
}
