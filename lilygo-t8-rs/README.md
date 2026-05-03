# lilygo-t8-rs

Rust firmware for the LilyGO T8 ESP32-S2 (aka T-Display S2) with ST7789 display and MLX90640 thermal camera support.

<img width="320" height="341" alt="thermal" src="https://github.com/user-attachments/assets/3df30444-f379-4385-b1b4-dedb7bc8f9ad" />

## Hardware

- **Board**: LilyGO T8 ESP32-S2 (135×240 ST7789 SPI display, 4MB flash, 8MB PSRAM)
- **Sensor**: Pimoroni PIM365 (MLX90640 32×24 thermal camera, I2C)
- **Pinout**: see [`doc/board_reference.md`](doc/board_reference.md)

## Quick Start

```bash
# Install esp toolchain (one-time)
cargo install espup
espup install

# Build, flash, and monitor (from this directory, or use -p from workspace root)
cargo run --release
```

## Setup

### Linker

The `.cargo/config.toml` specifies the Xtensa GCC linker path. If `espup` installed your toolchain to a different location, update the `linker` line:

```toml
[target.xtensa-esp32s2-none-elf]
linker = "/path/to/xtensa-esp32s2-elf-gcc"
```

The default path follows the `espup` convention:
```
$HOME/.rustup/toolchains/esp/xtensa-esp-elf/<version>/xtensa-esp-elf/bin/xtensa-esp32s2-elf-gcc
```

### Camera wiring

| PIM365 Qwiic | GPIO  | Header 2 Pin |
|-------------|-------|-------------|
| SDA (blue)  | GPIO39 | 12 |
| SCL (yellow)| GPIO40 | 13 |
| VCC (red)   | —     | 1 (VBUS) |
| GND (black) | —     | 2 |

## Apps

Navigate with the BOOT button: **short press** cycles or toggles, **long press** selects or returns to menu.

### Main Menu

Shows a list of available apps. Short press moves the selection cursor, long press launches the highlighted app. The firmware returns here after exiting any app.

### Screen Test

Displays color bars (RED, GREEN, BLUE, CYAN, MAGENTA, YELLOW, WHITE), a hue gradient, and a checkerboard pattern.

| Input | Action |
|-------|--------|
| Short press | Cycle backlight brightness: 0% → 25% → 50% → 75% → 100% |
| Long press | Return to menu |

### Thermal Camera

MLX90640 32×24 live thermal view with ironbow colormap. The overlay bar shows min/max temperature range, ambient temperature, frame rate, and I2C error count.

| Input | Action |
|-------|--------|
| Short press | Cycle frame rate (2/4/8/16/32 Hz) |
| Long press | Return to menu |

> **Note:** The I2C bus is consumed by the camera driver on entry and not returned on exit. Re-entering this app after leaving requires a reboot.
