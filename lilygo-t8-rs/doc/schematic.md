# ESP32-S2 Display Board — GPIO Pin Assignment Reference

This document lists **GPIO pin assignments and their connected functions** extracted from the schematic.

---

# ESP32-S2 GPIO Mapping

## Control & Boot

| GPIO   | Function             | Notes             |
| ------ | -------------------- | ----------------- |
| GPIO0  | BOOT / Download mode | Button to GND     |
| GPIO46 | Input only           | Internal pulldown |
| GPIO45 | VDD_SPI select       | Power config      |

---

## UART (USB Programming)

| GPIO            | Signal | Connected To |
| --------------- | ------ | ------------ |
| U0TXD (GPIO43*) | TX     | CH340C RXD   |
| U0RXD (GPIO44*) | RX     | CH340C TXD   |

(*logical mapping; actual pin names used in schematic: U0TXD/U0RXD)

---

## SPI Flash (Internal Bus)

| GPIO   | Signal | Device    |
| ------ | ------ | --------- |
| GPIO29 | SPICS0 | Flash CS  |
| GPIO30 | SPICLK | Flash CLK |
| GPIO32 | SPID   | MOSI      |
| GPIO31 | SPIQ   | MISO      |
| GPIO28 | SPIWP  | WP        |
| GPIO27 | SPIHD  | HOLD      |

---

## External SPI Device (PSAM)

| GPIO   | Signal | Device       |
| ------ | ------ | ------------ |
| GPIO29 | SPICS1 | External CS  |
| GPIO30 | SPICLK | Shared clock |
| GPIO32 | SPID   | Data         |
| GPIO31 | SPIQ   | Data         |
| GPIO28 | SPIWP  | Data         |
| GPIO27 | SPIHD  | Data         |

---

## Display (ST7789)

| GPIO   | Signal  | Notes         |
| ------ | ------- | ------------- |
| GPIO33 | MOSI    | SPI data      |
| GPIO34 | CS      | Chip select   |
| GPIO36 | CLK     | SPI clock     |
| GPIO37 | MISO    | (optional)    |
| GPIO38 | RESET   | Display reset |
| GPIO39 | DC / RS | Data/command  |

---

## TF Card (MicroSD)

| GPIO  | Signal     |
| ----- | ---------- |
| GPIO1 | DATA lines |
| GPIO2 | DATA lines |
| GPIO3 | DATA lines |
| GPIO4 | DATA lines |
| GPIO5 | DATA lines |
| GPIO6 | DATA lines |
| GPIO7 | DATA lines |
| GPIO8 | DATA lines |

(Note: SDIO multiplexed across multiple GPIOs)

---

## Additional SPI (FSPI / Secondary Mapping)

| GPIO   | Function |
| ------ | -------- |
| GPIO9  | FSPIHD   |
| GPIO10 | FSPICS0  |
| GPIO11 | FSPID    |
| GPIO12 | FSPICLK  |
| GPIO13 | FSPIQ    |

---

## General Purpose IO (Headers)

| GPIO   | Notes                      |
| ------ | -------------------------- |
| GPIO14 | Available                  |
| GPIO17 | Available                  |
| GPIO18 | Available                  |
| GPIO19 | Used (peripheral / header) |
| GPIO20 | Used (USB / peripheral)    |
| GPIO21 | Available                  |

---

## USB Interface Signals

| GPIO   | Signal |
| ------ | ------ |
| GPIO19 | D+     |
| GPIO20 | D-     |

(Through CH340C bridge)

---

## Buttons

| GPIO    | Function     |
| ------- | ------------ |
| GPIO0   | Boot button  |
| CHIP_PU | Reset button |

---

## Summary by Function

### SPI Bus

* GPIO27–GPIO32

### Secondary SPI (FSPI)

* GPIO9–GPIO13

### UART

* U0TXD / U0RXD

### Display

* GPIO33–GPIO39

### SD Card

* GPIO1–GPIO8

---

# Notes

* Some GPIOs are multiplexed across multiple peripherals.
* SPI bus is shared between flash, display, and external device.
* GPIO46 is input-only and cannot drive outputs.
* Boot configuration depends on GPIO0 state during reset.

---

# End of GPIO Assignment Document
