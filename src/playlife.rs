mod life;
mod counter;

use counter::Counter;
use life::life;

use crate::{Playfield, SwRng, nanorand::Rng};

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

pub type Grid = [[u8; 5]; 5];

pub struct Player {
    playfield: Playfield,
    grid: Grid,
    state: State,
    rng: SwRng,
}

impl Player {
    pub fn new(playfield: Playfield, rng: SwRng) -> Self {
        let grid = Grid::default();
        let state = State::Initing;
        Self { playfield, grid, state, rng }
    }

    pub async fn step(&mut self, button_a: bool, button_b: bool) {
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
                        life(&mut self.grid);
                        self.playfield.update(&self.grid);
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
        self.playfield.show().await;
    }

    /// Return `true` iff the grid contains no 1 pixels.
    fn done(&self) -> bool {
        self.grid == Grid::default()
    }

    /// Complement each cell in grid and update playfield.
    fn flip(&mut self) {
        for r in &mut self.grid {
            for cell in r {
                *cell = !*cell;
            }
        }
        self.playfield.update(&self.grid);
    }

    /// Randomize each cell in grid and update playfield.
    fn randomize(&mut self) {
        for r in &mut self.grid {
            for cell in r {
                *cell = self.rng.generate();
            }
        }
        self.playfield.update(&self.grid);
    }
}
