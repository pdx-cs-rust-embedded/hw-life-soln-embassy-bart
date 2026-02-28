//! Play Conway's Game of Life on a TFT display —
//! Embassy version.

#![no_std]
#![no_main]

mod playfield;
mod playlife;
mod spi_display;
use playfield::Playfield;
use playlife::{Player, life_async};
use spi_display::DirectInterface;

#[cfg(feature = "defmt")]
use defmt_rtt as _;
use panic_probe as _;

use microbit_bsp::{self, *, embassy_nrf::*};
pub use embassy_time::{Duration, Instant, Timer};

use embassy_futures::join::join;
use nanorand::{self, SeedableRng, Pcg64 as SwRng};

pub const FRAME_PERIOD: Duration = Duration::from_millis(33);

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
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    let board = Microbit::default();
    let rng = make_rng(board.rng).await;

    bind_interrupts!(struct SpimIrqs {
        SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
    });
    let mut spim_cfg = spim::Config::default();
    spim_cfg.frequency = spim::Frequency::M32;
    let spi_bus = spim::Spim::new_txonly(
        board.spi3,
        SpimIrqs,
        board.p13,
        board.p15,
        spim_cfg,
    );
    let dc  = gpio::Output::new(board.p8,  gpio::Level::Low,  gpio::OutputDrive::Standard);
    let cs  = gpio::Output::new(board.p1,  gpio::Level::High, gpio::OutputDrive::Standard);
    let rst = gpio::Output::new(board.p9,  gpio::Level::High, gpio::OutputDrive::Standard);
    let raw_display = mipidsi::Builder::new(
            mipidsi::models::GC9A01,
            DirectInterface::new(spi_bus, dc, cs),
        )
        .orientation(mipidsi::options::Orientation::new()
            .rotate(mipidsi::options::Rotation::Deg180))
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(rst)
        .init(&mut embassy_time::Delay)
        .unwrap();

    let mut playfield = Playfield::new(raw_display).await;
    let button_a = board.btn_a;
    let button_b = board.btn_b;

    let mut player = Player::new(rng);
    let mut grid_a = [[0u8; 120]; 120];
    let mut grid_b = [[0u8; 120]; 120];

    loop {
        let deadline = Instant::now() + FRAME_PERIOD;
        #[cfg(feature = "frame-timing")]
        let frame_start = Instant::now();

        let ba = button_a.is_low();
        let bb = button_b.is_low();

        player.advance(ba, bb, &mut grid_a);

        if player.is_running() {
            join(
                life_async(&grid_a, &mut grid_b),
                playfield.show(&grid_a, deadline),
            ).await;
            core::mem::swap(&mut grid_a, &mut grid_b);
        } else {
            playfield.show(&grid_a, deadline).await;
        }

        #[cfg(feature = "frame-timing")]
        player.log_frame_time(frame_start);
    }
}
