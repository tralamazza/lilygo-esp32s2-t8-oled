use crate::hw::{BacklightControl, ButtonInput, Delay};

use mipidsi::models::ST7789;

pub type Color = <ST7789 as mipidsi::models::Model>::ColorFormat;

mod menu;
mod screen_test;
pub mod thermal;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    Menu,
    ScreenTest,
    ThermalCamera,
}

impl AppKind {
    pub fn name(&self) -> &'static str {
        match self {
            AppKind::Menu => "Main Menu",
            AppKind::ScreenTest => "Screen Test",
            AppKind::ThermalCamera => "Thermal Camera",
        }
    }

    pub fn selectable() -> &'static [AppKind] {
        &[AppKind::ScreenTest, AppKind::ThermalCamera]
    }

    pub fn run<D, B, BL>(
        self,
        display: &mut D,
        backlight: &mut BL,
        button: &B,
        delay: &mut Delay,
        backlight_level: &mut u8,
    ) -> AppKind
    where
        D: embedded_graphics::prelude::DrawTarget<Color = Color>,
        D::Error: core::fmt::Debug,
        B: ButtonInput,
        BL: BacklightControl,
    {
        match self {
            AppKind::Menu => menu::run(display, backlight, button, delay, *backlight_level),
            AppKind::ScreenTest => {
                screen_test::run(display, backlight, button, delay, backlight_level)
            }
            AppKind::ThermalCamera => AppKind::Menu,
        }
    }
}
