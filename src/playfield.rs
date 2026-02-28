use crate::{Instant, Timer, playlife::Grid};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::Rgb565,
    prelude::RgbColor,
    primitives::Rectangle,
};

const ALIVE: Rgb565 = Rgb565::GREEN;
const DEAD: Rgb565 = Rgb565::BLACK;

pub struct Playfield<const NR: usize, const NC: usize, D> {
    display: D,
    frame: Grid<NR, NC>,
}

impl<const NR: usize, const NC: usize, D: DrawTarget<Color = Rgb565>>
    Playfield<NR, NC, D>
{
    pub fn new(mut display: D) -> Self {
        display.clear(DEAD).ok();
        Self { display, frame: [[0u8; NC]; NR] }
    }

    pub fn update(&mut self, grid: &Grid<NR, NC>) {
        self.frame = *grid;
    }

    pub async fn show(&mut self, deadline: Instant) {
        let cell_w = 240 / NC;
        let cell_h = 240 / NR;
        let bounds = Rectangle::new(Point::zero(), Size::new(240, 240));
        let display = &mut self.display;
        let frame = &self.frame;
        display
            .fill_contiguous(
                &bounds,
                (0..240_usize).flat_map(move |py| {
                    (0..240_usize).map(move |px| {
                        if frame[py / cell_h][px / cell_w] != 0 {
                            ALIVE
                        } else {
                            DEAD
                        }
                    })
                }),
            )
            .ok();
        Timer::at(deadline).await;
    }
}
