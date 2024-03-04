use crate::{
    SwRng,
    TICK,
    nanorand::Rng,
    microbit_bsp::LedMatrix,
    display::{Frame, Bitmap, Brightness},
};

pub struct Playfield {
    display: LedMatrix,
    rng: SwRng,
    frame: Frame<5, 5>,
}

impl Playfield {
    pub fn new(mut display: LedMatrix, rng: SwRng) -> Self {
        display.clear();
        display.set_brightness(Brightness::MAX);
        Self {
            display,
            rng,
            frame: Frame::empty(),
        }
    }

    pub fn randomize(&mut self) {
        let bitmaps = core::array::from_fn(|_| {
            let r: u8 = self.rng.generate();
            Bitmap::new(r, 5)
        });
        self.frame = Frame::new(bitmaps);
    }

    pub async fn display(&mut self) {
        self.display.apply(self.frame);
        self.display.display(self.frame, TICK).await;
    }
}
