//! AD5328 driver example
#![no_std]
#![no_main]

use ad5328::{Ad5328, Ad5328Config, Channel, VDD};
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_time::{fixed_point::FixedPoint, rate::*};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
    sio::Sio,
    spi,
    watchdog::Watchdog,
};

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

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // let mut led_pin = pins.led.into_push_pull_output();

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_sclk = pins.gpio2.into_mode::<gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio3.into_mode::<gpio::FunctionSpi>();
    let _spi_miso = pins.gpio4.into_mode::<gpio::FunctionSpi>();
    let spi_cs = pins.gpio5.into_push_pull_output();

    // Create an SPI driver instance for the SPI0 device
    let spi = spi::Spi::<_, _, 8>::new(pac.SPI0);

    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        &embedded_hal::spi::MODE_1,
    );

    let config = Ad5328Config {
        // To use Vdd as the voltage reference for all channels of the DAC
        vdd: (VDD::VddAsRef, VDD::VddAsRef),
        ..Default::default()
    };

    let mut dac = Ad5328::init(spi, spi_cs, config).unwrap();

    loop {
        info!("on!");
        // led_pin.set_high().unwrap();
        dac.set_channel(Channel::from(0), 4095).unwrap();
        delay.delay_ms(500);
        info!("off!");
        // led_pin.set_low().unwrap();
        dac.set_channel(Channel::from(0), 0).unwrap();
        delay.delay_ms(500);
    }
}

// End of file
