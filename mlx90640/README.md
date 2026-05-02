# mlx90640

`no_std` driver for the Melexis MLX90640 32×24 far-infrared thermal camera.

Ported from the [official Melexis C library](https://github.com/melexis/mlx90640-library).
Uses `embedded-hal` 1.0 I2C, with optional `libm` for float math.

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

let mut temps = [0.0f32; 768];
cam.generate_image(&mut temps).unwrap();
// temps contains corrected temperatures — bad pixels already interpolated

let ambient = cam.ambient_temperature();
```

`generate_image()` blocks until new frame data is available from the sensor,
returns corrected temperatures, and stores the ambient temperature internally.

## API

```rust
pub struct Mlx90640<I2C> { /* ... */ }

impl<I2C: embedded_hal::i2c::I2c> Mlx90640<I2C> {
    /// Loads EEPROM calibration and initializes driver.
    pub fn new(i2c: I2C) -> Result<Self, Error<I2C>>;

    /// Sets frame rate. Returns error on I2C failure.
    pub fn set_frame_rate(&mut self, rate: FrameRate) -> Result<(), Error<I2C>>;

    /// Blocks until new data ready, reads pixel RAM, calculates temperatures,
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
}
```

Calibration fails if more than 4 broken, 4 outlier, or 4 total bad pixels are found,
or if any bad pixels are adjacent.

## Frame rates

```rust
pub enum FrameRate {
    Half     = 0,  // 0.5 Hz
    One      = 1,  // 1 Hz
    Two      = 2,  // 2 Hz (default)
    Four     = 3,  // 4 Hz
    Eight    = 4,  // 8 Hz
    Sixteen  = 5,  // 16 Hz
    ThirtyTwo = 6, // 32 Hz
    SixtyFour = 7, // 64 Hz
}
```

## Cargo features

| Feature | Default | Description |
|---------|---------|-------------|
| `libm`  | yes     | Use `libm` for `sqrtf`/`powf`/`fabsf` in `no_std` |

Disable `libm` (`default-features = false`) if your target provides these intrinsics
(e.g. ESP32-S2 via compiler-builtins).
