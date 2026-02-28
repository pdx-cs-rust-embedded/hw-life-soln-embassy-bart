# life3: Life with MB2 and TFT display in Embassy
Bart Massey and Claude Code 2026

This crate is a demo of Conway's Life using the MB2 and a
GC9A01A SPI display. It is written with `embassy`,
`embassy-nrf` and `microbit-bsp`.

## Build and Run

You will need the usual MB2 setup and a way to connect the
display to the MB2.  Wire the display as follows:

| MB2 | DIS |
| --- | --- |
| e09 | RST |
| e01 | CS  |
| e08 | DC  |
| e15 | SDA |
| e13 | SCL |
| GND | GND |
| VCC | VCC |

Run with `cargo run --release --features=frame-rate`
to start the program. This will produce Conway's life on
the 480×480 round display with 2×2 square pixels. It will
also print the frame rate on the console every 100 frames.

Currently this code achieves around 20 FPS.

## Notes

This must use `SPIM3`, which is the only SPI interface to
support 32Mbps transfers.

The version of `microbit-bsp` used here currently is patched
to make this code work. Attempts to upstream things are
underway.

## Acknowledgments

Claude Code was used extensively in later commits on this
project.

## License

This work is made available under the "Apache 2.0 or MIT
License". See the file `LICENSE.txt` in this distribution for
license terms.
