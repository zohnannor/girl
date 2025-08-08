#![expect(unsafe_code, reason = "unsafe code is used for SDL bindings")]
//! TODO docs

use core::mem;
use std::{sync::mpsc, thread};

use sdl2_sys::{
    SDL_Event, SDL_EventType, SDL_INIT_GAMECONTROLLER, SDL_Init, SDL_PollEvent,
    SDL_Quit, Uint32,
};
use tracing::error;

use crate::{Error, event::Event, util::get_sdl2_error};

/// TODO docs
const SDL_INIT_SUCCESS: libc::c_int = 0;

/// TODO docs
const SDL_QUIT: Uint32 = SDL_EventType::SDL_QUIT as Uint32;

/// TODO docs
const SDL_CONTROLLERDEVICEADDED: Uint32 =
    SDL_EventType::SDL_CONTROLLERDEVICEADDED as Uint32;

/// TODO docs
const NO_MORE_EVENTS: i32 = 0;

/// TODO docs
#[expect(unreachable_pub, dead_code, reason = "will make public when safe")]
#[derive(Debug)]
pub struct ControllerSystem {
    /// TODO docs
    ev_rx: mpsc::Receiver<Event>,
}

impl ControllerSystem {
    /// TODO docs
    ///
    /// # Errors
    ///
    /// TODO docs
    #[expect(unreachable_pub, dead_code, reason = "will make public when safe")]
    #[inline]
    pub fn new() -> Result<Self, Error> {
        // SAFETY: SDL_Init is called with `SDL_INIT_GAMECONTROLLER` which comes
        //         from SDL2 itself.
        let ok = unsafe { SDL_Init(SDL_INIT_GAMECONTROLLER) };

        if ok <= SDL_INIT_SUCCESS {
            error!(err = ?get_sdl2_error(), "SDL_Init failed");
            return Err(Error::Sdl2Init);
        }

        let (tx, rx) = mpsc::channel();

        drop(
            thread::Builder::new()
                .name("girl-sdl2".to_owned())
                .spawn(move || Self::poll_events(&tx))
                .map_err(Error::RuntimeInit)?,
        );

        Ok(Self { ev_rx: rx })
    }

    /// TODO docs
    #[expect(clippy::single_call_fn, reason = "pretty big chung of logic")]
    fn poll_events(tx: &mpsc::Sender<Event>) -> Result<(), Error> {
        let mut event = mem::MaybeUninit::<SDL_Event>::uninit();
        loop {
            // SAFETY: todo!()
            if unsafe { SDL_PollEvent(event.as_mut_ptr()) } == NO_MORE_EVENTS {
                continue; // TODO
            }

            // SAFETY: todo!()
            let event = unsafe { event.assume_init() };
            // SAFETY: todo!()
            let ty = unsafe { event.type_ };

            match ty {
                SDL_QUIT => {
                    tx.send(Event::Quit).map_err(Error::Send)?;
                    break; // TODO shutdown
                }
                SDL_CONTROLLERDEVICEADDED => {
                    tx.send(Event::ControllerConnected).map_err(Error::Send)?;
                }
                _ => {
                    // TODO
                }
            }
        }

        Ok(())
    }
}

impl Drop for ControllerSystem {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: TODO: figure out if this is really safe.
        unsafe {
            SDL_Quit();
        }
    }
}
