use crate::{Instant, Timer, playlife::Grid};
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};

const ALIVE: Rgb565 = Rgb565::GREEN;
const DEAD: Rgb565 = Rgb565::BLACK;

/// Convert an Rgb565 color to its big-endian byte pair for the display.
fn color_be(c: Rgb565) -> [u8; 2] {
    let v = ((c.r() as u16) << 11) | ((c.g() as u16) << 5) | (c.b() as u16);
    v.to_be_bytes()
}

/// Trait for displays that support direct row-by-row pixel writing.
/// Implemented for `mipidsi::Display` in `spi_display.rs`.
pub trait RowWriter {
    fn begin_frame(&mut self);
    fn write_row(&mut self, row_bytes: &[u8]);
}

pub struct Playfield<const NR: usize, const NC: usize, D> {
    display: D,
    frame: Grid<NR, NC>,
}

impl<const NR: usize, const NC: usize, D: RowWriter> Playfield<NR, NC, D> {
    pub fn new(mut display: D) -> Self {
        let [dh, dl] = color_be(DEAD);
        let mut row = [0u8; 480];
        for i in 0..240 {
            row[i * 2] = dh;
            row[i * 2 + 1] = dl;
        }
        display.begin_frame();
        for _ in 0..240 {
            display.write_row(&row);
        }
        Self { display, frame: [[0u8; NC]; NR] }
    }

    pub fn update(&mut self, grid: &Grid<NR, NC>) {
        self.frame = *grid;
    }

    pub async fn show(&mut self, deadline: Instant) {
        let [ah, al] = color_be(ALIVE);
        let [dh, dl] = color_be(DEAD);
        let cell_w = 240 / NC;
        let cell_h = 240 / NR;
        let mut row_buf = [0u8; 480];

        self.display.begin_frame();
        for py in 0..240usize {
            let grid_row = py / cell_h;
            for px in 0..240usize {
                let alive = self.frame[grid_row][px / cell_w] != 0;
                row_buf[px * 2] = if alive { ah } else { dh };
                row_buf[px * 2 + 1] = if alive { al } else { dl };
            }
            self.display.write_row(&row_buf);
        }
        Timer::at(deadline).await;
    }
}
