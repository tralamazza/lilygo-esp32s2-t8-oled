#[cfg(not(feature = "libm"))]
unsafe extern "C" {
    fn sqrtf(x: f32) -> f32;
    fn fabsf(x: f32) -> f32;
}

#[cfg(feature = "libm")]
#[inline]
pub fn sqrt(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[cfg(not(feature = "libm"))]
#[inline]
pub fn sqrt(x: f32) -> f32 {
    unsafe { sqrtf(x) }
}

#[cfg(feature = "libm")]
#[inline]
pub fn abs(x: f32) -> f32 {
    libm::fabsf(x)
}

#[cfg(not(feature = "libm"))]
#[inline]
pub fn abs(x: f32) -> f32 {
    unsafe { fabsf(x) }
}

/// Fast `2.0_f32.powi(exp)` via direct IEEE 754 exponent manipulation.
///
/// Valid for `exp` in `[-126, 127]` (subnormals/infinity outside this range).
/// All exponents used in this crate fall within this bound.
#[inline]
pub fn pow2f(exp: i32) -> f32 {
    f32::from_bits((exp.wrapping_add(127) as u32) << 23)
}
