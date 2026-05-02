use crate::apps::Color;
use crate::hw::{
    BacklightControl, ButtonEvent, ButtonInput, Delay, wait_for_event, wait_for_release,
};
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text},
};
use mlx90640::{FrameRate, Mlx90640};

const W: i32 = 135;
const H: i32 = 240;
const IMG_W: i32 = 120;
const IMG_H: i32 = 160;
const X_OFF: i32 = (W - IMG_W) / 2;
const SENSOR_W: usize = 32;
const SENSOR_H: usize = 24;
const SENSOR_N: usize = SENSOR_W * SENSOR_H;
const FRAME_RATES: [FrameRate; 5] = [
    FrameRate::Two,
    FrameRate::Four,
    FrameRate::Eight,
    FrameRate::Sixteen,
    FrameRate::ThirtyTwo,
];

type Colormap = [Color; 256];

fn make_ironbow() -> Colormap {
    let mut map = [Color::BLACK; 256];
    for i in 0u8..=255 {
        let idx = i as usize;
        let (r, g, b) = if i < 32 {
            (0, 0, (i as u32 * 8).min(255) as u8)
        } else if i < 64 {
            let v = ((i - 32) as u32 * 8).min(255) as u8;
            (0, v, 255)
        } else if i < 96 {
            let v = ((i - 64) as u32 * 8).min(255) as u8;
            (v, 255, 255)
        } else if i < 128 {
            (255, 255u8.saturating_sub((i - 96) * 8), 0u8)
        } else if i < 160 {
            let v = ((i - 128) as u32 * 8).min(255) as u8;
            (255, 255u8.saturating_sub(v), 0)
        } else if i < 200 {
            (255, (i - 160) * 6 / 2 + 64, 0)
        } else {
            let v = ((i - 200) as u32 * 4).min(255) as u8;
            (255, 255, v)
        };
        map[idx] = Color::new(r, g, b);
    }
    map
}

fn temp_to_idx(temp: f32, min: f32, max: f32) -> u8 {
    let range = max - min;
    if range < 0.1 {
        return 128;
    }
    let t = (temp - min) / range;
    (t.clamp(0.0, 1.0) * 255.0) as u8
}

#[allow(dead_code)]
fn gaussian_3x3(temps: &mut [f32; SENSOR_N]) {
    let orig = *temps;
    for row in 1..SENSOR_H - 1 {
        for col in 1..SENSOR_W - 1 {
            let i = row * SENSOR_W + col;
            let s = orig[i - SENSOR_W - 1] * 0.0625
                + orig[i - SENSOR_W] * 0.125
                + orig[i - SENSOR_W + 1] * 0.0625
                + orig[i - 1] * 0.125
                + orig[i] * 0.25
                + orig[i + 1] * 0.125
                + orig[i + SENSOR_W - 1] * 0.0625
                + orig[i + SENSOR_W] * 0.125
                + orig[i + SENSOR_W + 1] * 0.0625;
            temps[i] = s;
        }
    }
}

fn f32_to_int_frac(v: f32) -> (i16, u8) {
    let int = v as i16;
    let frac = ((v - int as f32).abs() * 10.0 + 0.5) as u8;
    (int, if frac > 9 { 9 } else { frac })
}

struct FmtBuf {
    buf: [u8; 32],
    len: usize,
}

impl FmtBuf {
    fn new() -> Self {
        Self {
            buf: [0; 32],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl Write for FmtBuf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s_bytes = s.as_bytes();
        let remaining = &mut self.buf[self.len..];
        let n = s_bytes.len().min(remaining.len());
        remaining[..n].copy_from_slice(&s_bytes[..n]);
        self.len += n;
        if n < s_bytes.len() {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }
}

pub fn run<D, B, BL, I2C>(
    display: &mut D,
    backlight: &mut BL,
    button: &B,
    _delay: &mut Delay,
    backlight_level: &mut u8,
    i2c_opt: &mut Option<I2C>,
) -> super::AppKind
where
    D: DrawTarget<Color = Color>,
    D::Error: core::fmt::Debug,
    B: ButtonInput,
    BL: BacklightControl,
    I2C: embedded_hal::i2c::I2c,
{
    let i2c = match i2c_opt.take() {
        Some(bus) => bus,
        None => {
            display.clear(Color::BLACK).unwrap();
            let font = MonoTextStyle::new(&FONT_6X10, Color::RED);
            let _ = Text::with_alignment(
                "Camera unavailable",
                Point::new(W / 2, H / 2),
                font,
                Alignment::Center,
            )
            .draw(display);
            Delay::wait_ms(2000);
            return super::AppKind::Menu;
        }
    };

    let mut fr_idx: usize = 2;
    let mut fr = FRAME_RATES[fr_idx];

    let mut camera = match Mlx90640::new(i2c) {
        Ok(c) => c,
        Err(e) => {
            display.clear(Color::BLACK).unwrap();
            let font = MonoTextStyle::new(&FONT_6X10, Color::RED);
            let msg = match e {
                mlx90640::Error::I2cError(_) => "I2C err - chk wiring",
                _ => "Calibration err",
            };
            let _ = Text::with_alignment(msg, Point::new(W / 2, H / 2), font, Alignment::Center)
                .draw(display);
            let font2 = MonoTextStyle::new(&FONT_6X10, Color::WHITE);
            let _ = Text::with_alignment(
                "Hold btn to exit",
                Point::new(W / 2, H / 2 + 20),
                font2,
                Alignment::Center,
            )
            .draw(display);
            loop {
                if let ButtonEvent::LongPress = wait_for_event(button) {
                    wait_for_release(button);
                    return super::AppKind::Menu;
                }
            }
        }
    };

    backlight.set_brightness(*backlight_level);

    let colormap = make_ironbow();

    let mut temps = [0.0f32; SENSOR_N];
    let mut err_count: u16 = 0;

    let font_w = MonoTextStyle::new(&FONT_6X10, Color::WHITE);
    let mut row_buf = [Color::BLACK; W as usize];

    loop {
        match camera.generate_image(&mut temps) {
            Ok(()) => {
                let (min_t, max_t) = temps.iter().fold((f32::MAX, f32::MIN), |(mn, mx), &t| {
                    (if t < mn { t } else { mn }, if t > mx { t } else { mx })
                });

                let mut indices = [[0u8; SENSOR_W]; SENSOR_H];
                for row in 0..SENSOR_H {
                    for col in 0..SENSOR_W {
                        indices[row][col] = temp_to_idx(temps[row * SENSOR_W + col], min_t, max_t);
                    }
                }

                for y in 0..IMG_H {
                    let sensor_col = SENSOR_W - 1 - y as usize / 5;
                    for x in 0..IMG_W {
                        let sensor_row = x as usize / 5;
                        row_buf[(X_OFF + x) as usize] =
                            colormap[indices[sensor_row][sensor_col] as usize];
                    }
                    let area = Rectangle::new(Point::new(0, y), Size::new(W as u32, 1));
                    display
                        .fill_contiguous(&area, row_buf.iter().copied())
                        .unwrap();
                }

                Rectangle::new(
                    Point::new(0, IMG_H),
                    Size::new(W as u32, (H - IMG_H) as u32),
                )
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .fill_color(Color::BLACK)
                        .build(),
                )
                .draw(display)
                .unwrap();

                let (min_int, min_frac) = f32_to_int_frac(min_t);
                let (max_int, max_frac) = f32_to_int_frac(max_t);
                let amb = camera.ambient_temperature();
                let (amb_int, amb_frac) = f32_to_int_frac(amb);

                let mut buf1 = FmtBuf::new();
                let _ = write!(
                    buf1,
                    "{}.{} - {}.{}C A:{}.{}C",
                    min_int, min_frac, max_int, max_frac, amb_int, amb_frac,
                );

                let map_name = "iron";
                let mut buf2 = FmtBuf::new();
                let _ = write!(
                    buf2,
                    "{}  {}hz  e:{}",
                    map_name,
                    f32::from(fr) as u8,
                    err_count,
                );

                let rect_y = H - 28;

                let _ = Text::with_alignment(
                    buf1.as_str(),
                    Point::new(W / 2, rect_y + 10),
                    font_w,
                    Alignment::Center,
                )
                .draw(display);

                let _ = Text::with_alignment(
                    buf2.as_str(),
                    Point::new(W / 2, rect_y + 22),
                    font_w,
                    Alignment::Center,
                )
                .draw(display);
            }
            Err(_) => {
                err_count = err_count.saturating_add(1);
                Delay::wait_ms(100);
            }
        }

        match wait_for_event(button) {
            ButtonEvent::LongPress => {
                wait_for_release(button);
                return super::AppKind::Menu;
            }
            ButtonEvent::ShortPress => {
                fr_idx = (fr_idx + 1) % FRAME_RATES.len();
                fr = FRAME_RATES[fr_idx];
                let _ = camera.set_frame_rate(fr);
            }
            _ => {}
        }
    }
}
