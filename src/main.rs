//! Play Conway's Game of Life on the MB2 LED "display" —
//! Embassy version.

#![no_std]
#![no_main]

mod playfield;
mod playlife;
use playfield::Playfield;
use playlife::Player;

use defmt_rtt as _;
use panic_probe as _;

use microbit_bsp::{self, *, embassy_nrf::*};

use embassy_executor::Spawner;
use embassy_time::Duration;

use nanorand::{self, SeedableRng, Pcg64 as SwRng};

pub const TICK: Duration = Duration::from_millis(100u64);

async fn make_rng(board_rng: peripherals::RNG) -> SwRng {
    bind_interrupts!(struct Irqs {
        RNG => rng::InterruptHandler<peripherals::RNG>;
    });
    let mut hw_rng = rng::Rng::new(board_rng, Irqs);

    let mut seed = [0u8; 16];
    hw_rng.fill_bytes(&mut seed).await;
    core::mem::forget(hw_rng);

    let mut rng = SwRng::new_seed(1);
    rng.reseed(seed);
    rng
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let board = Microbit::default();
    let rng = make_rng(board.rng).await;
    let display = Playfield::new(board.display);
    let mut player = Player::new(display, rng);
    let button_a = board.btn_a;
    let button_b = board.btn_b;

    loop {
        player.step(button_a.is_low(), button_b.is_low()).await;
    }
}
