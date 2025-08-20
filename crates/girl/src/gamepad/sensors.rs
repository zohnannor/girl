//! Sensor data for a [`Gamepad`].

use sdl2::sensor::SensorType as SdlSensorType;

use crate::{Error, Gamepad};

/// Sensor data for a [`Gamepad`].
#[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
// TODO: Try remove on next Rust version update.
#[expect(clippy::allow_attributes, reason = "`#[expect]` doesn't work here")]
#[allow(
    clippy::multiple_inherent_impl,
    reason = "feature gated and documented"
)]
impl Gamepad {
    /// Query whether the gamepad has a specific sensor.
    #[must_use]
    #[inline]
    pub fn has_sensor(&self, sensor_type: Sensor) -> bool {
        self.gp.has_sensor(sensor_type.into_sdl())
    }

    /// Enables a [`Sensor`] on the [`Gamepad`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the sensor is not available or fails to
    /// enable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Sensor;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_sensor(Sensor::Gyroscope) {
    ///     gamepad.enable_sensor(Sensor::Gyroscope)?;
    ///     // read sensor data later
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    #[inline]
    pub fn enable_sensor(&self, sensor: Sensor) -> Result<(), Error> {
        self.gp
            .sensor_set_enabled(sensor.into_sdl(), true)
            .map_err(|err| Error::SdlError(err.to_string()))
    }

    /// Gets current [`Sensor`] data.
    ///
    /// You will need to enable the [`Sensor`] first using
    /// [`enable_sensor`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the [`Sensor`] is not available or fails to
    /// read.
    ///
    /// # Examples
    ///
    /// ```
    /// # use girl::Sensor;
    /// let mut girl = girl::Girl::new()?;
    /// # if girl.gamepad(0).is_some() {
    /// let mut gamepad = girl.gamepad(0).unwrap();
    ///
    /// if gamepad.has_sensor(Sensor::Gyroscope) {
    ///     gamepad.enable_sensor(Sensor::Gyroscope)?;
    ///     let [x, y, z] = gamepad.sensor(Sensor::Gyroscope)?;
    ///     // apply movement to a character, etc.
    /// }
    /// # }
    /// # Ok::<(), girl::Error>(())
    /// ```
    ///
    /// [`enable_sensor`]: Self::enable_sensor
    #[inline]
    pub fn sensor(&self, sensor: Sensor) -> Result<[f64; 3], Error> {
        let mut data = [0.; 3];
        self.gp
            .sensor_get_data(sensor.into_sdl(), &mut data)
            .map_err(|err| Error::SdlError(err.to_string()))?;
        Ok(data.map(|x| super::map(f64::from(x), 0.01, 1.)))
    }
}

/// Sensors available on [`Gamepad`]s.
#[cfg_attr(docsrs, doc(cfg(feature = "sensors")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "if gamepads get more sensors in the future, we'll add them in a \
              major update"
)]
pub enum Sensor {
    /// Unknown sensor type.
    Unknown,

    /// Gyroscope.
    Gyroscope,

    /// Gyroscope for left Joy-Con controller .
    LeftGyroscope,

    /// Gyroscope for right Joy-Con controller.
    RightGyroscope,

    /// Accelerometer.
    Accelerometer,

    /// Accelerometer for left Joy-Con controller.
    LeftAccelerometer,

    /// Accelerometer for right Joy-Con controller.
    RightAccelerometer,
}

impl Sensor {
    /// Converts from [`SdlSensorType`].
    #[must_use]
    #[inline]
    #[expect(clippy::single_call_fn, reason = "extracted conversion")]
    pub(crate) const fn from_sdl(sensor: SdlSensorType) -> Self {
        match sensor {
            SdlSensorType::Unknown => Self::Unknown,
            SdlSensorType::Gyroscope => Self::Gyroscope,
            SdlSensorType::LeftGyroscope => Self::LeftGyroscope,
            SdlSensorType::RightGyroscope => Self::RightGyroscope,
            SdlSensorType::Accelerometer => Self::Accelerometer,
            SdlSensorType::LeftAccelerometer => Self::LeftAccelerometer,
            SdlSensorType::RightAccelerometer => Self::RightAccelerometer,
        }
    }

    /// Converts to [`SdlSensorType`].
    #[must_use]
    #[inline]
    const fn into_sdl(self) -> SdlSensorType {
        match self {
            Self::Unknown => SdlSensorType::Unknown,
            Self::Gyroscope => SdlSensorType::Gyroscope,
            Self::LeftGyroscope => SdlSensorType::LeftGyroscope,
            Self::RightGyroscope => SdlSensorType::RightGyroscope,
            Self::Accelerometer => SdlSensorType::Accelerometer,
            Self::LeftAccelerometer => SdlSensorType::LeftAccelerometer,
            Self::RightAccelerometer => SdlSensorType::RightAccelerometer,
        }
    }
}
