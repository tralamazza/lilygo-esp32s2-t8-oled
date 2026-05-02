use crate::apps::{AppKind, Color};
use crate::hw::{
    BacklightControl, ButtonEvent, ButtonInput, Delay, wait_for_event, wait_for_release,
};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
};

const W: i32 = 135;
const H: i32 = 240;
const ITEM_H: i32 = 20;
const TOP_MARGIN: i32 = 10;

fn draw_menu<D>(display: &mut D, selection: usize)
where
    D: DrawTarget<Color = Color>,
    D::Error: core::fmt::Debug,
{
    display.clear(Color::BLACK).unwrap();

    let title_font = MonoTextStyle::new(&FONT_6X10, Color::WHITE);
    Text::with_alignment(
        "Select App",
        Point::new(W / 2, TOP_MARGIN),
        title_font,
        Alignment::Center,
    )
    .draw(display)
    .unwrap();

    let items = AppKind::selectable();
    for (i, app) in items.iter().enumerate() {
        let y = TOP_MARGIN + 16 + (i as i32 * ITEM_H);
        let is_selected = i == selection;

        if is_selected {
            let sel_style = PrimitiveStyleBuilder::new()
                .fill_color(Color::WHITE)
                .build();
            Rectangle::new(Point::new(4, y - 2), Size::new((W - 8) as u32, 14))
                .into_styled(sel_style)
                .draw(display)
                .unwrap();
        }

        let color = if is_selected {
            Color::BLACK
        } else {
            Color::WHITE
        };
        let font = MonoTextStyle::new(&FONT_6X10, color);
        Text::with_alignment(
            app.name(),
            Point::new(W / 2, y + 8),
            font,
            Alignment::Center,
        )
        .draw(display)
        .unwrap();
    }

    let hint_font = MonoTextStyle::new(&FONT_6X10, Color::WHITE);
    Text::with_alignment(
        "Tap: next  Hold: select",
        Point::new(W / 2, H - 10),
        hint_font,
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
    current_level: u8,
) -> AppKind
where
    D: DrawTarget<Color = Color>,
    D::Error: core::fmt::Debug,
    B: ButtonInput,
    BL: BacklightControl,
{
    backlight.set_brightness(current_level);

    let items = AppKind::selectable();
    let mut selection: usize = 0;

    draw_menu(display, selection);

    wait_for_release(button);

    loop {
        match wait_for_event(button) {
            ButtonEvent::ShortPress => {
                selection = (selection + 1) % items.len();
                draw_menu(display, selection);
            }
            ButtonEvent::LongPress => {
                wait_for_release(button);
                return items[selection];
            }
            ButtonEvent::None => {}
        }
    }
}
