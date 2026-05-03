use crate::calibration::CalibrationParams;
use crate::math;

fn get_median(values: &mut [f32], n: usize) -> f32 {
    values[..n].sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    if n.is_multiple_of(2) {
        (values[n / 2] + values[n / 2 - 1]) / 2.0
    } else {
        values[n / 2]
    }
}

fn is_pixel_bad(pixel: u16, params: &CalibrationParams) -> bool {
    for i in 0..5 {
        if pixel == params.outlier_pixels[i] || pixel == params.broken_pixels[i] {
            return true;
        }
    }
    false
}

pub(crate) fn correct_bad_pixels(
    pixels: &[u16; 5],
    to: &mut [f32],
    mode: u16,
    params: &CalibrationParams,
) {
    let mut bad_pixels_all = [0xFFFFu16; 5];
    let mut count = 0;

    for &pix in pixels.iter() {
        if pix != 0xFFFF {
            bad_pixels_all[count] = pix;
            count += 1;
        }
    }

    let mut ap = [0.0f32; 4];

    for &pix in bad_pixels_all.iter().take(count) {
        let line = (pix >> 5) as usize;
        let column = (pix as usize) - (line << 5);

        if mode == 1 {
            // Chess mode: median of diagonal neighbors
            if line == 0 {
                if column == 0 {
                    to[pix as usize] = to[33];
                } else if column == 31 {
                    to[pix as usize] = to[62];
                } else {
                    to[pix as usize] = (to[pix as usize + 31] + to[pix as usize + 33]) / 2.0;
                }
            } else if line == 23 {
                if column == 0 {
                    to[pix as usize] = to[705];
                } else if column == 31 {
                    to[pix as usize] = to[734];
                } else {
                    to[pix as usize] = (to[pix as usize - 33] + to[pix as usize - 31]) / 2.0;
                }
            } else if column == 0 {
                to[pix as usize] = (to[pix as usize - 31] + to[pix as usize + 33]) / 2.0;
            } else if column == 31 {
                to[pix as usize] = (to[pix as usize - 33] + to[pix as usize + 31]) / 2.0;
            } else {
                ap[0] = to[pix as usize - 33];
                ap[1] = to[pix as usize - 31];
                ap[2] = to[pix as usize + 31];
                ap[3] = to[pix as usize + 33];
                to[pix as usize] = get_median(&mut ap, 4);
            }
        } else {
            // Interleave mode: gradient-aware horizontal interpolation
            if column == 0 {
                to[pix as usize] = to[pix as usize + 1];
            } else if column == 1 || column == 30 {
                to[pix as usize] = (to[pix as usize - 1] + to[pix as usize + 1]) / 2.0;
            } else if column == 31 {
                to[pix as usize] = to[pix as usize - 1];
            } else if !is_pixel_bad(pix.wrapping_sub(2), params)
                && !is_pixel_bad(pix.wrapping_add(2), params)
            {
                ap[0] = to[pix as usize + 1] - to[pix as usize + 2];
                ap[1] = to[pix as usize - 1] - to[pix as usize - 2];
                if math::abs(ap[0]) > math::abs(ap[1]) {
                    to[pix as usize] = to[pix as usize - 1] + ap[1];
                } else {
                    to[pix as usize] = to[pix as usize + 1] + ap[0];
                }
            } else {
                to[pix as usize] = (to[pix as usize - 1] + to[pix as usize + 1]) / 2.0;
            }
        }
    }
}
