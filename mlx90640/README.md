# mlx90640

`no_std` driver for the Melexis MLX90640 32×24 far-infrared thermal camera.

Ported from the [official Melexis C library](https://github.com/melexis/mlx90640-library).
Uses `embedded-hal` 1.0 I2C and `libm` for `no_std` float math.

## Features

- EEPROM calibration parameter extraction
- Temperature calculation per pixel (chess pattern & interleave modes)
- Bad pixel detection — scans EEPROM for broken pixels (calibration word `0x0000`)
  and outlier flags (LSB set)
- Bad pixel correction — diagonal median interpolation (chess) or gradient-aware
  horizontal interpolation (interleave)

## Usage

```rust
use mlx90640::{FrameRate, Mlx90640};

let mut cam = Mlx90640::new(i2c).unwrap();
cam.set_frame_rate(FrameRate::Eight).unwrap();
cam.set_emissivity(0.95);
cam.set_tr(25.0);

let mut temps = [0.0f32; 768];
cam.generate_image(&mut temps).unwrap();
// temps contains corrected temperatures — bad pixels already interpolated

let ambient = cam.ambient_temperature();
```

`generate_image()` polls up to 2000 times (or until I2C error) waiting for a new frame,
then reads pixel RAM, calculates temperatures, and corrects bad pixels. Returns
`Error::Timeout` if no frame arrives within the poll limit. Stores the ambient temperature internally.

## API

```rust
pub struct Mlx90640<I2C> { /* ... */ }

impl<I2C: embedded_hal::i2c::I2c> Mlx90640<I2C> {
    /// Loads EEPROM calibration and initializes driver.
    /// Does not set a frame rate — call set_frame_rate() before generate_image().
    pub fn new(i2c: I2C) -> Result<Self, Error<I2C>>;

    /// Sets frame rate. Returns error on I2C failure.
    pub fn set_frame_rate(&mut self, rate: FrameRate) -> Result<(), Error<I2C>>;

    /// Sets emissivity (default 0.95).
    pub fn set_emissivity(&mut self, e: f32);

    /// Sets reflected temperature in °C (default 25.0).
    pub fn set_tr(&mut self, tr: f32);

    /// Polls for new frame (with timeout), reads pixel RAM, calculates temperatures,
    /// corrects bad pixels. Writes result into `dest` (768 f32 values).
    pub fn generate_image(&mut self, dest: &mut [f32; 768]) -> Result<(), Error<I2C>>;

    /// Ambient temperature from the last `generate_image()` call.
    /// Returns 25°C before the first frame.
    pub fn ambient_temperature(&self) -> f32;
}
```

## Error

```rust
pub enum Error<I2C: embedded_hal::i2c::I2c> {
    I2cError(I2C::Error),
    TooManyBrokenPixels,
    TooManyOutlierPixels,
    TooManyBadPixels,
    AdjacentBadPixels,
    FrameDataError,
    Timeout,
}
```

Calibration fails if more than 4 broken, 4 outlier, or 4 total bad pixels are found,
or if any bad pixels are adjacent.

## Frame rates

```rust
pub enum FrameRate {
    Half     = 0,  // 0.5 Hz (factory default after POR)
    One      = 1,  // 1 Hz
    Two      = 2,  // 2 Hz
    Four     = 3,  // 4 Hz
    Eight    = 4,  // 8 Hz
    Sixteen  = 5,  // 16 Hz
    ThirtyTwo = 6, // 32 Hz
    SixtyFour = 7, // 64 Hz
}
```
