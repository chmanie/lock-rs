//! Polls and debounces a button press
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use debounced_button::{Button, ButtonConfig, ButtonState};
use defmt::*;
use defmt_rtt as _;
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;

use rp_pico;

use rp_pico::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

// For rust-analyzer we disable the #[entry] macro
#[cfg_attr(not(test), entry)]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let btn_pin = pins.gpio17.into_pull_up_input();
    let config = ButtonConfig::default();
    let mut button = Button::new(btn_pin, 1_000, config);

    loop {
        // Ideally this would be run in a timer interrupt
        button.poll();
        delay.delay_ms(1);
        match button.read() {
            ButtonState::Press => {
                info!("Button pressed!");
            }
            ButtonState::LongPress => {
                info!("Button pressed long");
            }
            _ => {}
        }
    }
}
