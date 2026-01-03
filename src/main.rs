//! Play Conway's Game of Life on the MB2 LED "display" —
//! Embassy version.
//!
//! The `backlight` feature enables driving an external RGB
//! LED as a "backlight" that goes through a RGB color cycle
//! independent of the game.

#![no_std]
#![no_main]

mod playfield;
mod playlife;
mod leds;
use playfield::Playfield;
use playlife::Player;
#[cfg(feature = "backlight")]
use leds::cycle_leds;

#[cfg(feature = "backlight")]
use defmt::unwrap;
use defmt_rtt as _;
use panic_probe as _;

use microbit_bsp::{self, *, embassy_nrf::*, embassy_time::{Duration, Timer}};

use embassy_executor::{self, Spawner};

use nanorand::{self, SeedableRng, Pcg64 as SwRng};

pub const TICK: Duration = Duration::from_millis(100u64);

async fn make_rng(board_rng: Peri<'_, peripherals::RNG>) -> SwRng {
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

    #[cfg(feature = "backlight")] {
        macro_rules! led {
            ($pin:expr) => {
                gpio::Output::new(
                    $pin,
                    gpio::Level::Low,
                    gpio::OutputDrive::Standard,
                )
            };
        }

        let red = led!(board.p9);   // GPIO1
        let green = led!(board.p8);   // GPIO2
        let blue = led!(board.p16);   // GPIO3
        unwrap!(_spawner.spawn(cycle_leds([red, green, blue])));
    }

    loop {
        player.step(button_a.is_low(), button_b.is_low()).await;
    }
}
