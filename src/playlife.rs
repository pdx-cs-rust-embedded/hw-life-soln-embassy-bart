mod life;
mod counter;

use counter::Counter;
use life::life;

use crate::{Playfield, SwRng, nanorand::Rng, Instant, FRAME_PERIOD};
use crate::playfield::RowWriter;

/// Play states.
pub enum State {
    /// State to start a new randomized board.
    Initing,
    /// State to wait `remaining` ticks before continuing.
    Paused { remaining: Counter },
    /// State to complement the board.
    Flipping,
    /// State to step the game. Will only accept the "flip"
    /// button if `last_flip` is 0.
    Running { last_flip: Counter },
}

pub type Grid<const NR: usize, const NC: usize> = [[u8; NC]; NR];

pub struct Player<const NR: usize, const NC: usize, D> {
    playfield: Playfield<NR, NC, D>,
    current: Grid<NR, NC>,
    next_grid: Grid<NR, NC>,
    state: State,
    rng: SwRng,
    #[cfg(feature = "frame-timing")]
    frame_count: u32,
    #[cfg(feature = "frame-timing")]
    frame_total_us: u32,
}

impl<const NR: usize, const NC: usize, D: RowWriter>
    Player<NR, NC, D>
{
    pub fn new(playfield: Playfield<NR, NC, D>, rng: SwRng) -> Self {
        let state = State::Initing;
        Self {
            playfield,
            current: [[0u8; NC]; NR],
            next_grid: [[0u8; NC]; NR],
            state,
            rng,
            #[cfg(feature = "frame-timing")]
            frame_count: 0,
            #[cfg(feature = "frame-timing")]
            frame_total_us: 0,
        }
    }

    pub async fn step(&mut self, button_a: bool, button_b: bool) {
        #[cfg(feature = "frame-timing")]
        let frame_start = Instant::now();
        let deadline = Instant::now() + FRAME_PERIOD;
        self.state = match self.state {
            State::Initing => {
                self.randomize();
                State::Running { last_flip: Counter(0) }
            }
            State::Running { .. } if self.done() => {
                State::Paused { remaining: Counter(5) }
            }
            State::Running { ref mut last_flip } => {
                match (button_a, button_b) {
                    (true, _) => State::Initing,
                    (_, true) if last_flip.is_zero() => State::Flipping,
                    _ => {
                        life(&self.current, &mut self.next_grid);
                        core::mem::swap(&mut self.current, &mut self.next_grid);
                        self.playfield.update(&self.current);
                        State::Running { last_flip: last_flip.decr() }
                    }
                }
            }
            State::Paused { ref mut remaining } => {
                if !remaining.is_zero() {
                    State::Paused { remaining: remaining.decr() }
                } else {
                    State::Initing
                }
            }
            State::Flipping => {
                self.flip();
                State::Running { last_flip: Counter(5) }
            }
        };
        self.playfield.show(deadline).await;
        #[cfg(feature = "frame-timing")]
        {
            let elapsed_us = (Instant::now() - frame_start).as_micros() as u32;
            self.frame_total_us = self.frame_total_us.saturating_add(elapsed_us);
            self.frame_count += 1;
            if self.frame_count >= 100 {
                let avg_us = self.frame_total_us / self.frame_count;
                defmt::info!(
                    "frame avg: {}ms (~{}fps)",
                    avg_us / 1000,
                    1_000_000u32 / avg_us,
                );
                self.frame_count = 0;
                self.frame_total_us = 0;
            }
        }
    }

    /// Return `true` iff the grid contains no live cells.
    fn done(&self) -> bool {
        self.current.iter().all(|row| row.iter().all(|&c| c == 0))
    }

    /// Complement each cell in current and update playfield.
    fn flip(&mut self) {
        for r in &mut self.current {
            for cell in r {
                *cell = if *cell == 0 { 1 } else { 0 };
            }
        }
        self.playfield.update(&self.current);
    }

    /// Randomize each cell in current and update playfield.
    fn randomize(&mut self) {
        for r in &mut self.current {
            for cell in r {
                *cell = self.rng.generate::<bool>() as u8;
            }
        }
        self.playfield.update(&self.current);
    }
}
