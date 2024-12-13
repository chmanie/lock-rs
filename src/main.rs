#![no_std]
#![no_main]

use embassy_executor::{Executor, Spawner};
use embassy_futures::join::join;
use embassy_futures::join::join3;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Output;
// use embassy_rp::block::ImageDef;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::peripherals::{PIN_12, PIN_13, PIN_14, PIN_15, USB};
use embassy_rp::usb;
use embassy_rp::{bind_interrupts, peripherals::PIO0, pio};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcAcmState};
use embassy_usb::{Builder, Config};
use pio_proc::pio_asm;

use {defmt_rtt as _, panic_probe as _};

use static_cell::StaticCell;

// #[link_section = ".start_block"]
// #[used]
// pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static mut CORE1_STACK: Stack<4096> = Stack::new();
pub static PUBSUB: PubSubChannel<CriticalSectionRawMutex, usize, 4, 16, 1> = PubSubChannel::new();

#[embassy_executor::task]
async fn publish(pio0: PIO0, pin12: PIN_12, pin13: PIN_13, pin14: PIN_14, pin15: PIN_15) {
    let pio::Pio {
        mut common,
        mut sm0,
        ..
    } = pio::Pio::new(pio0, Irqs);

    let prg = pio_asm!(
        "
        pull block
        out pins, 4
        "
    );
    let pin0 = common.make_pio_pin(pin12);
    let pin1 = common.make_pio_pin(pin13);
    let pin2 = common.make_pio_pin(pin14);
    let pin3 = common.make_pio_pin(pin15);
    sm0.set_pin_dirs(pio::Direction::Out, &[&pin0, &pin1, &pin2, &pin3]);
    let mut cfg = pio::Config::default();
    cfg.set_out_pins(&[&pin0, &pin1, &pin2, &pin3]);
    cfg.use_program(&common.load_program(&prg.program), &[]);
    sm0.set_config(&cfg);
    sm0.set_enable(true);

    let mut chan: usize = 0;
    let change_publisher = PUBSUB.publisher().unwrap();

    loop {
        Timer::after_millis(1).await;

        change_publisher.publish_immediate(15 - chan);

        chan = (chan + 1) % 16;
    }
}

#[embassy_executor::task]
async fn run_usb(usb0: USB) {
    let usb_driver = usb::Driver::new(usb0, Irqs);
    let mut usb_config = Config::new(0xf569, 0x1);
    usb_config.manufacturer = Some("ACME");
    usb_config.product = Some("FOO");
    usb_config.serial_number = Some("12345678");
    usb_config.max_power = 500;
    usb_config.device_class = 0xEF;
    usb_config.device_sub_class = 0x02;
    usb_config.device_protocol = 0x01;
    usb_config.composite_with_iads = true;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut logger_state = CdcAcmState::new();

    let mut usb_builder = Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    let usb_logger = CdcAcmClass::new(&mut usb_builder, &mut logger_state, 64);

    let log_fut = embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, usb_logger);

    let mut usb = usb_builder.build();

    join(usb.run(), log_fut).await;
}

#[embassy_executor::task(pool_size = 16)]
async fn run_app(number: usize) {
    let fut1 = async {
        loop {
            Timer::after_millis(10).await;
        }
    };

    let fut2 = async {
        let mut subscriber = PUBSUB.subscriber().unwrap();
        loop {
            subscriber.next_message_pure().await;
        }
    };

    let fut3 = async {
        loop {
            log::info!("APP HEARTBEAT, CHAN {}", number);
            Timer::after_secs(4).await;
        }
    };

    join3(fut1, fut2, fut3).await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led0 = Output::new(p.PIN_26, Level::Low);
    let mut led1 = Output::new(p.PIN_27, Level::Low);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|sp| {
                for i in 0..16 {
                    sp.spawn(run_app(i)).unwrap();
                }
            });
        },
    );

    spawner
        .spawn(publish(p.PIO0, p.PIN_12, p.PIN_13, p.PIN_14, p.PIN_15))
        .unwrap();

    spawner.spawn(run_usb(p.USB)).unwrap();

    let mut i = 0_u8;
    loop {
        Timer::after_secs(5).await;
        log::info!("Heartbeat {}", i);
        i = i.wrapping_add(1);
    }
}
