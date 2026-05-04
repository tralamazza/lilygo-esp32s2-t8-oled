#![no_std]

mod bad_pixels;
mod calculations;
mod calibration;
mod driver;
mod math;
#[cfg(feature = "precompute")]
mod patterns;
mod types;

pub use driver::Mlx90640;
pub use types::{Error, FrameRate};

pub const WIDTH: usize = 32;
pub const HEIGHT: usize = 24;
pub const NUM_PIXELS: usize = 768;
pub const ADDRESS: u8 = 0x33;
