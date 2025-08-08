#![expect(unsafe_code, reason = "unsafe code is used for SDL bindings")]
//! TODO docs

use core::ffi::CStr;

use sdl2_sys::SDL_GetError;

/// TODO docs
#[expect(
    clippy::single_call_fn,
    reason = "common logic for getting SDL2 errors"
)]
pub(crate) fn get_sdl2_error() -> String {
    // SAFETY: todo!()
    let error = unsafe { SDL_GetError() };
    // SAFETY: todo!()
    unsafe { CStr::from_ptr(error) }.to_string_lossy().into_owned()
}
