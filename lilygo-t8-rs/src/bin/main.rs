#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use display_interface_spi::SPIInterface;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::CpuClock,
    gpio::{DriveMode, Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{self, SoftwareTimeout},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, TimerIFace},
    },
    main, spi,
    time::{Duration, Rate},
};
use lilygo_t8_rs::apps::AppKind;
use lilygo_t8_rs::hw::Delay;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let sclk = peripherals.GPIO36;
    let mosi = peripherals.GPIO35;

    let dc = Output::new(peripherals.GPIO37, Level::Low, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO34, Level::High, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO38, Level::Low, OutputConfig::default());
    let bl = peripherals.GPIO33;

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(5),
        })
        .unwrap();

    let mut channel0 = ledc.channel(channel::Number::Channel0, bl);
    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 50,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    let button = Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    );

    let spi = spi::master::Spi::new(
        peripherals.SPI2,
        spi::master::Config::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(spi::Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi);

    let spi_dev = match ExclusiveDevice::new(spi, cs, Delay) {
        Ok(d) => d,
        Err(_) => panic!("failed to create ExclusiveDevice"),
    };

    let di = SPIInterface::new(spi_dev, dc);

    let mut display = mipidsi::Builder::new(mipidsi::models::ST7789, di)
        .display_size(135, 240)
        .display_offset(52, 40)
        .reset_pin(rst)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();

    let i2c = master::I2c::new(
        peripherals.I2C0,
        master::Config::default()
            .with_frequency(Rate::from_khz(400))
            .with_software_timeout(SoftwareTimeout::Transaction(Duration::from_millis(100))),
    )
    .unwrap()
    .with_sda(peripherals.GPIO39)
    .with_scl(peripherals.GPIO40);

    let mut i2c_opt = Some(i2c);

    let mut delay = Delay;
    let mut backlight_level: u8 = 50;

    let mut current_app = AppKind::Menu;

    loop {
        if current_app == AppKind::ThermalCamera {
            current_app = lilygo_t8_rs::apps::thermal::run(
                &mut display,
                &mut channel0,
                &button,
                &mut delay,
                &mut backlight_level,
                &mut i2c_opt,
            );
        } else {
            current_app = current_app.run(
                &mut display,
                &mut channel0,
                &button,
                &mut delay,
                &mut backlight_level,
            );
        }
    }
}
