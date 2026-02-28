use crate::{
    TICK,
    microbit_bsp::LedMatrix,
    display::{Frame, Brightness},
    playlife::Grid,
};

pub struct Playfield<const NR: usize, const NC: usize> {
    display: LedMatrix,
    frame: Frame<NR, NC>,
}

impl Playfield<5, 5> {
    pub fn new(mut display: LedMatrix) -> Self {
        display.clear();
        display.set_brightness(Brightness::MAX);
        Self {
            display,
            frame: Frame::empty(),
        }
    }

    pub fn update(&mut self, grid: &Grid<5, 5>) {
        for (r, row) in grid.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if *cell == 0 {
                    self.frame.unset(r, c);
                } else {
                    self.frame.set(r, c);
                }
            }
        }
    }

    pub async fn show(&mut self) {
        self.display.apply(self.frame);
        self.display.display(self.frame, TICK).await;
    }
}
