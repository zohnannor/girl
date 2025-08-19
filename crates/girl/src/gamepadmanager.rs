//! Gamepad manager and connection handling.
//!
//! This module provides the main interface for detecting and managing
//! connected [`Gamepad`]s.

use core::fmt;

use crate::{Error, Event, gamepad::Gamepad};

/// Main gamepad manager.
///
/// Handles initialization, event processing, and gamepad connection management.
/// The name "`Girl`" is an acronym for "Gamepad Input Rust Library".
///
/// # Examples
///
/// ```
/// let mut girl = girl::Girl::new()?;
/// # if girl.gamepad(0).is_some() {
/// let mut gamepad = girl.gamepad(0).unwrap();
///
/// loop {
///     girl.update();
///     if !gamepad.connected()
///         && let Some(gp) = girl.gamepad(0)
///     {
///         gamepad = gp;
///     }
///     // check buttons, sensors, etc.
///     # break;
/// }
/// # }
/// # Ok::<(), girl::Error>(())
/// ```
pub struct Girl {
    /// SDL2 game controller subsystem.
    gcs: sdl2::GameControllerSubsystem,
    /// SDL2 joystick subsystem.
    jcs: sdl2::JoystickSubsystem,
    /// SDL2 event pump for processing input events.
    event_pump: sdl2::EventPump,
}

impl fmt::Debug for Girl {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Girl")
            .field("gamepad_subsystem", &self.gcs)
            .field("joystick_subsystem", &self.jcs)
            .field("event_pump", &"...")
            .finish()
    }
}

impl Girl {
    /// Initializes a new gamepad input manager.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL2 or its controller subsystems fail to
    /// initialize.
    #[inline]
    pub fn new() -> Result<Self, Error> {
        let sdl2 = sdl2::init().map_err(Error::Sdl2Init)?;
        let gamepad_subsys = sdl2.game_controller().map_err(Error::Sdl2Init)?;
        let joystick_subsys = sdl2.joystick().map_err(Error::Sdl2Init)?;
        let event_pump = sdl2.event_pump().map_err(Error::Sdl2Init)?;

        Ok(Self { gcs: gamepad_subsys, jcs: joystick_subsys, event_pump })
    }

    /// Polls for the next available input [`Event`].
    ///
    /// Returns [`None`] if no events are currently available.
    #[must_use]
    #[inline]
    pub fn event(&mut self) -> Option<Event> {
        self.event_pump.poll_event().as_ref().and_then(Event::from_sdl)
    }

    /// Waits for and returns the next input [`Event`].
    ///
    /// Blocks until an [`Event`] is available.
    #[must_use]
    #[inline]
    pub fn event_blocking(&mut self) -> Event {
        loop {
            if let Some(ev) = Event::from_sdl(&self.event_pump.wait_event()) {
                return ev;
            }
        }
    }

    /// Gathers pending input events from [`Gamepad`] devices.
    ///
    /// Should be called regularly in your application's main loop, as otherwise
    /// the [`Gamepad`] will report same inputs over and over again.
    #[inline]
    pub fn update(&mut self) {
        self.event_pump.pump_events();
        debug_assert!(self.gcs.event_state(), "unhandled events");
    }

    /// Returns an iterator over all connected [`Gamepad`]s.
    #[inline]
    pub const fn gamepads_connected(&self) -> ConnectedGamepads<'_> {
        ConnectedGamepads { gcs: &self.gcs, jcs: &self.jcs, idx: 0 }
    }

    /// Gets a specific [`Gamepad`] by its `index`.
    ///
    /// Returns [`None`] if no [`Gamepad`] is connected at the given `index`.
    #[must_use]
    #[inline]
    pub fn gamepad(&self, index: u32) -> Option<Gamepad> {
        let gc = self.gcs.open(index).ok()?;
        let js = self.jcs.open(index).ok()?;
        Gamepad::from_sdl(gc, js)
    }

    // /// Returns the latest [`TouchpadEvent`], if any.
    // #[must_use]
    // #[inline]
    // pub fn touchpad(&mut self) -> Option<TouchpadEvent> {
    //     let mut tp = None;
    //     while let Some(event) = self.event() {
    //         if let Some(tpn) = TouchpadEvent::from_event(event) {
    //             tp = Some(tpn);
    //         }
    //     }
    //     tp
    // }
}

/// Iterator over all connected [`Gamepad`]s.
///
/// Can be obtained from [`Girl::gamepads_connected`].
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ConnectedGamepads<'girl> {
    /// Reference to the game controller subsystem.
    gcs: &'girl sdl2::GameControllerSubsystem,
    /// Reference to the joystick subsystem.
    jcs: &'girl sdl2::JoystickSubsystem,
    /// Current index being iterated.
    idx: u32,
}

impl Iterator for ConnectedGamepads<'_> {
    type Item = Gamepad;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // skip over non-gamepads
        while !self.gcs.is_game_controller(self.idx) {
            self.idx = self.idx.checked_add(1)?;
        }
        let gc = self.gcs.open(self.idx).ok()?;
        let js = self.jcs.open(self.idx).ok()?;
        let gamepad = Gamepad::from_sdl(gc, js);
        self.idx = self.idx.checked_add(1)?;
        gamepad
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for ConnectedGamepads<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.gcs.num_joysticks().unwrap_or(0) as usize
    }
}
