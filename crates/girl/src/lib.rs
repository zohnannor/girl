#![cfg_attr(doc, doc = include_str!("../README.md"))]
//! ## Feature flags
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod event;
mod gamepad;
mod gamepadmanager;

mod unused {
    //! Only used for documentation.
    use document_features as _;
    // Not actually used, dev-dependency for example/demo.
    #[cfg(test)]
    use tracing_subscriber as _;
}

use alloc::string::String;

// TODO: logging
#[cfg(feature = "tracing")]
use tracing as _;

#[cfg(feature = "sensors")]
#[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
pub use crate::gamepad::sensors::Sensor;
#[cfg(feature = "touchpad")]
#[cfg_attr(docsrs, doc(cfg(feature = "touchpad")))]
pub use crate::gamepad::touchpad::{
    TouchpadAction, TouchpadEvent, TouchpadState,
};
pub use crate::{
    event::Event,
    gamepad::{
        Gamepad, PowerLevel,
        input::{Button, Stick, Trigger},
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
