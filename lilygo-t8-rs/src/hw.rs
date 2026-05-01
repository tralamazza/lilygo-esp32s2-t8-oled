use embedded_hal::delay::DelayNs;
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerSpeed;
use esp_hal::time::{Duration, Instant};

pub struct Delay;

impl DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        let us = (ns as u64).div_ceil(1000);
        let start = Instant::now();
        while start.elapsed() < Duration::from_micros(us) {}
    }
}

impl Delay {
    pub fn wait_ms(ms: u64) {
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(ms) {}
    }
}

pub trait BacklightControl {
    fn set_brightness(&mut self, pct: u8);
}

impl<'a, S: TimerSpeed> BacklightControl for esp_hal::ledc::channel::Channel<'a, S> {
    fn set_brightness(&mut self, pct: u8) {
        let _ = ChannelIFace::set_duty(self, pct);
    }
}

pub trait ButtonInput {
    fn pressed(&self) -> bool;
}

impl<'d> ButtonInput for esp_hal::gpio::Input<'d> {
    fn pressed(&self) -> bool {
        self.is_low()
    }
}

pub enum ButtonEvent {
    None,
    ShortPress,
    LongPress,
}

pub fn wait_for_event<B: ButtonInput>(button: &B) -> ButtonEvent {
    if !button.pressed() {
        Delay::wait_ms(20);
        return ButtonEvent::None;
    }

    let press_start = Instant::now();

    while button.pressed() {
        if press_start.elapsed() >= Duration::from_millis(500) {
            return ButtonEvent::LongPress;
        }
        Delay::wait_ms(20);
    }

    ButtonEvent::ShortPress
}

pub fn wait_for_release<B: ButtonInput>(button: &B) {
    while button.pressed() {
        Delay::wait_ms(20);
    }
}
