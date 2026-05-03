#[cfg(not(feature = "libm"))]
unsafe extern "C" {
    fn sqrtf(x: f32) -> f32;
    fn powf(x: f32, y: f32) -> f32;
    fn fabsf(x: f32) -> f32;
}

#[cfg(feature = "libm")]
pub fn sqrt(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[cfg(not(feature = "libm"))]
pub fn sqrt(x: f32) -> f32 {
    unsafe { sqrtf(x) }
}

#[cfg(feature = "libm")]
pub fn pow(x: f32, y: f32) -> f32 {
    libm::powf(x, y)
}

#[cfg(not(feature = "libm"))]
pub fn pow(x: f32, y: f32) -> f32 {
    unsafe { powf(x, y) }
}

#[cfg(feature = "libm")]
pub fn abs(x: f32) -> f32 {
    libm::fabsf(x)
}

#[cfg(not(feature = "libm"))]
pub fn abs(x: f32) -> f32 {
    unsafe { fabsf(x) }
}
