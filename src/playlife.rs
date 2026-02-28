mod counter;
mod life;

use counter::Counter;
pub use life::life_async;

use crate::{SwRng, nanorand::Rng};

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

pub struct Player<const NR: usize, const NC: usize> {
    state: State,
    rng: SwRng,
    #[cfg(feature = "frame-timing")]
    frame_count: u32,
    #[cfg(feature = "frame-timing")]
    frame_total_us: u32,
}

impl<const NR: usize, const NC: usize> Player<NR, NC> {
    pub fn new(rng: SwRng) -> Self {
        Self {
            state: State::Initing,
            rng,
            #[cfg(feature = "frame-timing")]
            frame_count: 0,
            #[cfg(feature = "frame-timing")]
            frame_total_us: 0,
        }
    }

    /// Advance the state machine, modifying `grid` as needed
    /// (randomize, flip, etc.). Does NOT compute life — that
    /// is handled externally via `life_async`.
    pub fn advance(&mut self, button_a: bool, button_b: bool, grid: &mut Grid<NR, NC>) {
        self.state = match self.state {
            State::Initing => {
                self.randomize(grid);
                State::Running {
                    last_flip: Counter(0),
                }
            }
            State::Running { .. } if Self::done(grid) => State::Paused {
                remaining: Counter(5),
            },
            State::Running { ref mut last_flip } => match (button_a, button_b) {
                (true, _) => State::Initing,
                (_, true) if last_flip.is_zero() => State::Flipping,
                _ => State::Running {
                    last_flip: last_flip.decr(),
                },
            },
            State::Paused { ref mut remaining } => {
                if !remaining.is_zero() {
                    State::Paused {
                        remaining: remaining.decr(),
                    }
                } else {
                    State::Initing
                }
            }
            State::Flipping => {
                Self::flip(grid);
                State::Running {
                    last_flip: Counter(5),
                }
            }
        };
    }

    /// Return `true` iff currently in the Running state.
    pub fn is_running(&self) -> bool {
        matches!(self.state, State::Running { .. })
    }

    #[cfg(feature = "frame-timing")]
    pub fn log_frame_time(&mut self, start: crate::Instant) {
        let elapsed_us = (crate::Instant::now() - start).as_micros() as u32;
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

    /// Return `true` iff the grid contains no live cells.
    fn done(grid: &Grid<NR, NC>) -> bool {
        grid.iter().all(|row| row.iter().all(|&c| c == 0))
    }

    /// Complement each cell in the grid.
    fn flip(grid: &mut Grid<NR, NC>) {
        for r in grid {
            for cell in r {
                *cell = if *cell == 0 { 1 } else { 0 };
            }
        }
    }

    /// Randomize each cell in the grid.
    fn randomize(&mut self, grid: &mut Grid<NR, NC>) {
        for r in grid {
            for cell in r {
                *cell = self.rng.generate::<bool>() as u8;
            }
        }
    }
}
