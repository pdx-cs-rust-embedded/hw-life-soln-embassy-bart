use crate::{Instant, Timer, playlife::Grid};
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};

const ALIVE: Rgb565 = Rgb565::GREEN;
const DEAD: Rgb565 = Rgb565::BLACK;

const CHUNK_ROWS: usize = 20;
const CHUNK_BYTES: usize = CHUNK_ROWS * 480;

/// Convert an Rgb565 color to its big-endian byte pair for the display.
fn color_be(c: Rgb565) -> [u8; 2] {
    let v = ((c.r() as u16) << 11) | ((c.g() as u16) << 5) | (c.b() as u16);
    v.to_be_bytes()
}

/// Pixel colors for rendering.
struct PixelColors {
    alive_hi: u8,
    alive_lo: u8,
    dead_hi: u8,
    dead_lo: u8,
}

/// Trait for displays that support direct row-by-row pixel writing.
/// Implemented for `mipidsi::Display` in `spi_display.rs`.
pub trait RowWriter {
    fn begin_frame(&mut self);
    async fn write_row(&mut self, row_bytes: &[u8]);
}

pub struct Playfield<const NR: usize, const NC: usize, D> {
    display: D,
}

impl<const NR: usize, const NC: usize, D: RowWriter> Playfield<NR, NC, D> {
    pub async fn new(mut display: D) -> Self {
        let [dh, dl] = color_be(DEAD);
        let mut row = [0u8; 480];
        for i in 0..240 {
            row[i * 2] = dh;
            row[i * 2 + 1] = dl;
        }
        display.begin_frame();
        for _ in 0..240 {
            display.write_row(&row).await;
        }
        Self { display }
    }

    pub async fn show(&mut self, grid: &Grid<NR, NC>, deadline: Instant) {
        let colors = PixelColors {
            alive_hi: color_be(ALIVE)[0],
            alive_lo: color_be(ALIVE)[1],
            dead_hi: color_be(DEAD)[0],
            dead_lo: color_be(DEAD)[1],
        };
        let cell_w = 240 / NC;
        let cell_h = 240 / NR;
        let n_chunks = 240 / CHUNK_ROWS;

        let mut buf_a = [0u8; CHUNK_BYTES];
        let mut buf_b = [0u8; CHUNK_BYTES];

        self.display.begin_frame();

        // Fill first chunk into buf_a (no DMA yet)
        fill_chunk(grid, 0, cell_w, cell_h, &colors, &mut buf_a);

        for chunk in 1..n_chunks {
            let (send_buf, fill_buf) = if chunk % 2 == 1 {
                (&buf_a[..], &mut buf_b)
            } else {
                (&buf_b[..], &mut buf_a)
            };
            // Overlap: DMA sends previous chunk while CPU fills next
            embassy_futures::join::join(
                self.display.write_row(send_buf),
                async {
                    fill_chunk(grid, chunk, cell_w, cell_h, &colors, fill_buf);
                },
            ).await;
        }
        // Send last chunk
        let last = if n_chunks % 2 == 1 { &buf_a[..] } else { &buf_b[..] };
        self.display.write_row(last).await;

        Timer::at(deadline).await;
    }
}

fn fill_chunk<const NR: usize, const NC: usize>(
    frame: &Grid<NR, NC>,
    chunk: usize,
    cell_w: usize,
    cell_h: usize,
    colors: &PixelColors,
    buf: &mut [u8; CHUNK_BYTES],
) {
    let py_start = chunk * CHUNK_ROWS;
    for row_in_chunk in 0..CHUNK_ROWS {
        let py = py_start + row_in_chunk;
        let grid_row = py / cell_h;
        let offset = row_in_chunk * 480;
        for px in 0..240usize {
            let alive = frame[grid_row][px / cell_w] != 0;
            buf[offset + px * 2] = if alive { colors.alive_hi } else { colors.dead_hi };
            buf[offset + px * 2 + 1] = if alive { colors.alive_lo } else { colors.dead_lo };
        }
    }
}
