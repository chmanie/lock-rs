//! Reads the value of the first fader of the 16n faderbank using monome ii (i2c)
#![no_std]
#![no_main]

// https://www.thonk.co.uk/wp-content/uploads/2017/07/Stereo-Thonkiconn-Datasheetimage.jpg
// 3 TIP SDA
// 2 RING CLK
// 1 GND GND

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};
use mii::{Faderbank, Mii};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
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

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure two pins as being I²C, not GPIO
    let sda_pin = pins.gpio16.into_mode::<bsp::hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio17.into_mode::<bsp::hal::gpio::FunctionI2C>();

    // Create the I²C driver, using the two pre-configured pins. This will fail
    // at compile time if the pins are in the wrong mode, or if this I²C
    // peripheral isn't available on these pins!
    let i2c = bsp::hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock,
    );

    let mii = Mii::new(i2c);
    let mut faderbank = Faderbank::new(&mii);
    let mut fader_value = 0u16;

    loop {
        info!("on!");
        if let Ok(value) = faderbank.read_fader(0) {
            fader_value = value;
        }
        info!("fader 1 value: {}", fader_value);
        delay.delay_ms(500);
    }
}

// End of file
