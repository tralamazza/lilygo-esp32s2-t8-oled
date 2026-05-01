use crate::apps::Color;
use crate::hw::{
    BacklightControl, ButtonEvent, ButtonInput, Delay, wait_for_event, wait_for_release,
};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
};

const W: i32 = 135;
const BAR_H: i32 = 18;
const LEVELS: [u8; 5] = [0, 25, 50, 75, 100];

fn color_bar<D>(display: &mut D, y: i32, color: Color, label: &str, text_color: Color)
where
    D: DrawTarget<Color = Color>,
    D::Error: core::fmt::Debug,
{
    let style = PrimitiveStyleBuilder::new().fill_color(color).build();
    Rectangle::new(Point::new(0, y), Size::new(W as u32, BAR_H as u32))
        .into_styled(style)
        .draw(display)
        .unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, text_color);
    Text::with_alignment(
        label,
        Point::new(W / 2, y + BAR_H / 2 + 2),
        text_style,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();
}

pub fn run<D, B, BL>(
    display: &mut D,
    backlight: &mut BL,
    button: &B,
    _delay: &mut Delay,
    backlight_level: &mut u8,
) -> super::AppKind
where
    D: DrawTarget<Color = Color>,
    D::Error: core::fmt::Debug,
    B: ButtonInput,
    BL: BacklightControl,
{
    display.clear(Color::BLACK).unwrap();

    let white = Color::WHITE;
    let black = Color::BLACK;

    color_bar(display, 0, Color::RED, "RED", white);
    color_bar(display, BAR_H, Color::GREEN, "GREEN", black);
    color_bar(display, BAR_H * 2, Color::BLUE, "BLUE", white);
    color_bar(display, BAR_H * 3, Color::CYAN, "CYAN", black);
    color_bar(display, BAR_H * 4, Color::MAGENTA, "MAGENTA", white);
    color_bar(display, BAR_H * 5, Color::YELLOW, "YELLOW", black);
    color_bar(display, BAR_H * 6, white, "WHITE", black);

    let grad_y = BAR_H * 7;
    let grad_h = 40;
    for x in 0..W {
        let t = x as u8 * 2;
        let r = if t < 128 { 128u8.saturating_sub(t) } else { 0 };
        let g = if t < 128 { t } else { 255u8.saturating_sub(t) };
        let b = if t >= 128 { t.saturating_sub(128) } else { 0 };
        let color = Color::new(r, g, b);
        let style = PrimitiveStyle::with_stroke(color, 1);
        Line::new(Point::new(x, grad_y), Point::new(x, grad_y + grad_h))
            .into_styled(style)
            .draw(display)
            .unwrap();
    }

    let chk_sz = 10;
    let chk_y = grad_y + grad_h + 4;
    for r in 0..(30 / chk_sz) {
        for c in 0..(W / chk_sz) {
            let color = if (r + c) % 2 == 0 { white } else { black };
            let style = PrimitiveStyleBuilder::new().fill_color(color).build();
            Rectangle::new(
                Point::new(c * chk_sz, chk_y + r * chk_sz),
                Size::new(chk_sz as u32, chk_sz as u32),
            )
            .into_styled(style)
            .draw(display)
            .unwrap();
        }
    }

    let label_y = chk_y + 30 + 6;
    let font = MonoTextStyle::new(&FONT_6X10, white);
    Text::with_alignment(
        "HOLD BTN: EXIT",
        Point::new(W / 2, label_y),
        font,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();

    let mut level_idx = LEVELS
        .iter()
        .position(|&l| l == *backlight_level)
        .unwrap_or(2);
    backlight.set_brightness(LEVELS[level_idx]);

    wait_for_release(button);

    loop {
        match wait_for_event(button) {
            ButtonEvent::ShortPress => {
                level_idx = (level_idx + 1) % LEVELS.len();
                backlight.set_brightness(LEVELS[level_idx]);
                *backlight_level = LEVELS[level_idx];
            }
            ButtonEvent::LongPress => {
                return super::AppKind::Menu;
            }
            ButtonEvent::None => {}
        }
    }
}
