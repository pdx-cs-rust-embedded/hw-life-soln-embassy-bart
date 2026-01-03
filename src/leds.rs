use crate::{gpio, Timer, embassy_executor};

#[embassy_executor::task(pool_size = 1)]
pub async fn cycle_leds(
    mut leds: [gpio::Output<'static>; 3]
) {
    let mut count = 0;
    loop {
        leds[(count + 2) % 3].set_low();
        leds[count].set_high();
        count = (count + 1) % 3;
        Timer::after_millis(300).await;
    }
}
