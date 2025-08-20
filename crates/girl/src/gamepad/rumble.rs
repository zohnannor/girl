//! Rumble capabilities of a [`Gamepad`].

use core::time::Duration;

use crate::{Error, Gamepad};

/// Rumble capabilities of a [`Gamepad`].
#[cfg_attr(docsrs, doc(cfg(feature = "rumble")))]
// TODO: Try remove on next Rust version update.
#[expect(clippy::allow_attributes, reason = "`#[expect]` doesn't work here")]
#[allow(
    clippy::multiple_inherent_impl,
    reason = "feature gated and documented"
)]
impl Gamepad {
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
