//! TODO docs

mod controllersystem;
mod event;
mod util;

use std::{io, sync::mpsc};

/// TODO docs
#[non_exhaustive]
#[derive(Debug)]
enum Error {
    /// TODO docs
    #[expect(dead_code, reason = "not for inspecting")]
    RuntimeInit(io::Error),

    /// TODO docs
    Sdl2Init,

    /// TODO docs
    #[expect(dead_code, reason = "not for inspecting")]
    Send(mpsc::SendError<event::Event>),
}
