# life2: Life on the MicroBit v2 in Embassy
Bart Massey 2024

This crate is a demo of Conway's Life on the MicroBit v2. It
is written with `embassy`, `embassy-nrf` and `microbit-bsp`.

Compiling with `--features=backlight` after connecting a RGB
LED to pins 8, 9 and 16 on the MB2 edge connector (and
grounding the LED) will cause the LED to independently flash
red-green-blue in a cycle.

See also the branch `tft-display` here, which gets good
performance on a fancy TFT display.
