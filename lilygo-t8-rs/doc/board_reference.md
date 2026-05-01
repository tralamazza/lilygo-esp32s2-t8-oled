# LilyGO T8 ESP32-S2 — Board Reference

Also known as: **LilyGo T-Display S2**

## Board Photos & Pinout

- **Pinout diagram**: `doc/pinout.png`
- **Original schematic**: `doc/ESP32_S2-Display.pdf`
- **RIOT-OS board page**: https://api.riot-os.org/group__boards__esp32s2__lilygo__ttgo__t8.html
- **LilyGO schematic repo**: https://github.com/Xinyuan-LilyGO/LilyGo-T-Display-S2

## Key Hardware

| Feature | Detail |
|---------|--------|
| SoC | ESP32-S2 (Xtensa single-core, no FPU) |
| Flash | 4 MB (W25Q32) |
| PSRAM | 8 MB QSPI |
| Display | ST7789 135×240, SPI via FSPI |
| SD Card | microSD slot (SPI via HSPI, GPIO10-13) |
| USB | USB-C with CH340C UART bridge |
| Crystal | 32.768 kHz (GPIO15/16, switchable via DIP) |
| Buttons | BOOT (GPIO0), RST |

## Header 1 (left, 20-pin single row)

| Pin | Silk Label | GPIO | Notes |
|-----|-----------|------|-------|
| 1 | IO0 | **GPIO0** | BOOT button (to GND) |
| 2 | GPIO1 | **GPIO1** | TF Card DATA2 |
| 3 | GPIO2 | **GPIO2** | TF Card DATA3 |
| 4 | GPIO3 | **GPIO3** | TF Card CMD (strapping pin) |
| 5 | GPIO4 | **GPIO4** | TF Card VDD |
| 6 | GPIO5 | **GPIO5** | TF Card CLK |
| 7 | GPIO6 | **GPIO6** | TF Card VSS |
| 8 | GPIO7 | **GPIO7** | TF Card DATA0 |
| 9 | GPIO8 | **GPIO8** | TF Card DATA1 |
| 10 | GPIO16 | **GPIO16** | UART / ESD chip; free if 32k crystal DIP off |
| 11 | GPIO15 | **GPIO15** | UART / ESD chip; free if 32k crystal DIP off |
| 12 | SPI_MOSI | **GPIO32** | Shared SPI bus (display + flash + PSRAM) |
| 13 | SPI_CLK | **GPIO30** | Shared SPI bus |
| 14 | SPI_MISO | **GPIO31** | Shared SPI bus |
| 15 | GPIO17 | **GPIO17** | Free (DAC capable) |
| 16 | GPIO18 | **GPIO18** | Free (DAC capable) |
| 17 | GPIO19 | **GPIO19** | USB D+ (or free if USB DIP off) |
| 18 | GPIO20 | **GPIO20** | USB D- (or free if USB DIP off) |
| 19 | BAT | — | LiPo battery voltage |
| 20 | +5V | — | VBUS 5V from USB-C |

## Header 2 (right, 20-pin single row)

| Pin | Silk Label | GPIO | Notes |
|-----|-----------|------|-------|
| 1 | VBUS | — | USB 5V |
| 2 | GND | — | Ground |
| 3 | GPIO21 | **GPIO21** | Free |
| 4 | GND | — | Ground |
| 5 | V3V | — | Regulated 3.3V output |
| 6 | GND | — | Ground |
| 7 | — | — | NC |
| 8 | VDD3V3 | — | 3.3V from LDO (AP2112K) |
| 9 | VDD_SPI | — | Flash voltage select (set by GPIO45) |
| 10 | — | — | NC |
| 11 | — | — | NC |
| 12 | GPIO39 | **GPIO39** | Free (PWM capable) |
| 13 | GPIO40 | **GPIO40** | Free (PWM capable) |
| 14 | GPIO41 | **GPIO41** | Free (PWM capable) |
| 15 | GPIO42 | **GPIO42** | Free (PWM capable) |
| 16 | GPIO45 | **GPIO45** | VDD_SPI strap — DO NOT USE |
| 17 | GPIO46 | **GPIO46** | Input-only (internal pulldown) |
| 18 | — | — | NC |
| 19 | GND | — | Ground |
| 20 | — | — | NC |

## Truly Free GPIOs (no hardware conflicts)

| GPIO | Header | Pin |
|------|--------|-----|
| GPIO14 | test point only | — |
| GPIO17 | Header 1 | 15 |
| GPIO18 | Header 1 | 16 |
| GPIO21 | Header 2 | 3 |
| GPIO39 | Header 2 | 12 |
| GPIO40 | Header 2 | 13 |
| GPIO41 | Header 2 | 14 |
| GPIO42 | Header 2 | 15 |

## Current Firmware Peripheral Usage

| Peripheral | GPIOs | Notes |
|-----------|-------|-------|
| ST7789 display SPI | CLK=36, MOSI=35, CS=34, DC=37, RST=38 | FSPI (SPI2), 40 MHz, Mode 0 |
| Backlight PWM | 33 | LEDC Timer0, Channel0, 5 kHz |
| BOOT button | 0 | Input, internal pull-up, active low |
| I2C0 (MLX90640) | SDA=39, SCL=40 | 400 kHz |

## MLX90640 Camera Wiring (Pimoroni PIM365)

| PIM365 Qwiic | GPIO | Header | Pin | Label |
|-------------|------|--------|-----|-------|
| SDA (blue) | GPIO39 | Header 2 | 12 | GPIO39 |
| SCL (yellow) | GPIO40 | Header 2 | 13 | GPIO40 |
| VCC (red) | — | Header 2 | 1 | VBUS |
| GND (black) | — | Header 2 | 2 | GND |

The PIM365 onboard regulator accepts 3.0–5.5V. VBUS (5V) is used to avoid blocking the SD slot.

## Important Notes

**Strapping pins** — do not use as outputs at boot:
- GPIO3 (TF Card CMD)
- GPIO45 (VDD_SPI voltage select)
- GPIO46 (input-only, can read but not drive)

**DIP switches:**
- GPIO15/16: connect to 32k crystal by default. Can DIP-switch to free them.
- GPIO19/20: connect USB D+/D- to CH340C UART by default. Can DIP-switch to route USB OTG to SoC.

**Shared SPI bus:**
- FSPI (GPIO27-32): shared between internal flash, PSRAM, and display
- HSPI (GPIO9-13): dedicated to SD card slot; GPIO10-13 broken out on header

**No FPU:** ESP32-S2 uses software floating point. MLX90640 temperature calculation uses `libm` for math functions.

## References

- RIOT-OS board config: https://api.riot-os.org/group__boards__esp32s2__lilygo__ttgo__t8.html
- RIOT-OS pin definitions: https://github.com/RIOT-OS/RIOT/blob/master/boards/esp32s2-lilygo-ttgo-t8/include/periph_conf.h
- LilyGO GitHub: https://github.com/Xinyuan-LilyGO/LilyGo-T-Display-S2
- MLX90640 driver crate: https://crates.io/crates/mlx9064x
