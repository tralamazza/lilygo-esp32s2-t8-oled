# lilygo_rs

Rust firmware for the [LilyGO T8 ESP32-S2](https://github.com/Xinyuan-LilyGO/LilyGO-T-Embed) (ST7789 135x240 display, MLX90640 IR camera).

## Structure

```
lilygo_rs/
├── Cargo.toml            # workspace root
├── mlx90640/             # no_std MLX90640 IR camera driver
└── lilygo-t8-rs/         # target firmware (ESP32-S2)
```

## mlx90640 crate

`no_std` driver for the Melexis MLX90640 32×24 IR thermal camera, ported from the
[official C library](https://github.com/melexis/mlx90640-library).

```rust
use mlx90640::Mlx90640;

let mut camera = Mlx90640::new(i2c)?;
camera.set_frame_rate(FrameRate::Eight)?;

let mut temps = [0.0f32; 768];
camera.generate_image(&mut temps)?;
// temps now holds corrected temperatures with bad pixels interpolated
```

- **embedded-hal 1.0** I2C trait (`I2c`)
- **Bad pixel detection**: scans EEPROM for broken pixels (`0x0000`) and outlier flags
- **Bad pixel correction**: diagonal median (chess mode) or gradient-aware horizontal interpolation (interleave mode)

## Build

Requires the [ESP Rust toolchain](https://docs.esp-rs.org/book/installation/):

```sh
rustup override set esp
cargo build -p lilygo-t8-rs
```

Flash:

```sh
cargo run -p lilygo-t8-rs
```
