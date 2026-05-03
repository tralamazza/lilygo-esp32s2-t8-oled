use crate::calibration::{CalibrationParams, SCALE_ALPHA};
use crate::math;

pub(crate) fn get_vdd(frame_data: &[u16; 834], params: &CalibrationParams) -> f32 {
    let resolution_ram = (frame_data[832] >> 10) & 0x03;
    let resolution_correction = math::pow(2.0, params.resolution_ee as f32) / math::pow(2.0, resolution_ram as f32);
    (resolution_correction * frame_data[810] as i16 as f32 - params.vdd25 as f32) / params.kvdd as f32 + 3.3
}

pub(crate) fn get_ta(frame_data: &[u16; 834], params: &CalibrationParams) -> f32 {
    let vdd = get_vdd(frame_data, params);
    let ptat = frame_data[800] as i16 as f32;
    let ptat_art = (ptat / (ptat * params.alpha_ptat + frame_data[768] as i16 as f32)) * math::pow(2.0, 18.0);
    
    (ptat_art / (1.0 + params.kvptat * (vdd - 3.3)) - params.vptat25 as f32) / params.ktptat + 25.0
}

pub(crate) fn calculate_to(
    frame_data: &[u16; 834],
    params: &CalibrationParams,
    emissivity: f32,
    tr: f32,
    result: &mut [f32; 768],
) -> f32 {
    let subpage = frame_data[833];
    let vdd = get_vdd(frame_data, params);
    let ta = get_ta(frame_data, params);

    let mut ta4 = ta + 273.15;
    ta4 = ta4 * ta4;
    ta4 = ta4 * ta4;
    let mut tr4 = tr + 273.15;
    tr4 = tr4 * tr4;
    tr4 = tr4 * tr4;
    let ta_tr = tr4 - (tr4 - ta4) / emissivity;

    let kta_scale = math::pow(2.0, params.kta_scale as f32);
    let kv_scale = math::pow(2.0, params.kv_scale as f32);
    let alpha_scale = math::pow(2.0, params.alpha_scale as f32);

    let mut alpha_corr_r = [1.0f32; 4];
    alpha_corr_r[0] = 1.0 / (1.0 + params.ks_to[0] * 40.0);
    alpha_corr_r[2] = 1.0 + params.ks_to[1] * params.ct[2] as f32;
    alpha_corr_r[3] = alpha_corr_r[2] * (1.0 + params.ks_to[2] * (params.ct[3] - params.ct[2]) as f32);

    let gain = params.gain_ee as f32 / frame_data[778] as i16 as f32;

    let mode = (frame_data[832] & 0x1000) >> 5;

    let mut ir_data_cp = [0.0f32; 2];
    ir_data_cp[0] = frame_data[776] as i16 as f32 * gain;
    ir_data_cp[1] = frame_data[808] as i16 as f32 * gain;

    ir_data_cp[0] -= params.cp_offset[0] as f32
            * (1.0 + params.cp_kta * (ta - 25.0))
            * (1.0 + params.cp_kv * (vdd - 3.3));

    if mode == params.calibration_mode_ee as u16 {
        ir_data_cp[1] -= params.cp_offset[1] as f32
                * (1.0 + params.cp_kta * (ta - 25.0))
                * (1.0 + params.cp_kv * (vdd - 3.3));
    } else {
        ir_data_cp[1] -= (params.cp_offset[1] as f32 + params.il_chess_c[0])
                * (1.0 + params.cp_kta * (ta - 25.0))
                * (1.0 + params.cp_kv * (vdd - 3.3));
    }

    for pixel_number in 0..768u16 {
        let pixel = pixel_number as usize;
        let pix_i32 = pixel_number as i32;

        let il_pattern = pix_i32 / 32 - (pix_i32 / 64) * 2;
        let chess_pattern = il_pattern ^ (pix_i32 - (pix_i32 / 2) * 2);
        let conversion_pattern = ((pix_i32 + 2) / 4 - (pix_i32 + 3) / 4 + (pix_i32 + 1) / 4 - pix_i32 / 4)
            * (1 - 2 * il_pattern);

        let pattern = if mode == 0 { il_pattern } else { chess_pattern };

        if pattern as u16 == subpage {
            let mut ir_data = frame_data[pixel] as i16 as f32 * gain;

            let kta = params.kta[pixel] as f32 / kta_scale;
            let kv = params.kv[pixel] as f32 / kv_scale;
            ir_data -= params.offset[pixel] as f32 * (1.0 + kta * (ta - 25.0)) * (1.0 + kv * (vdd - 3.3));

            if mode != params.calibration_mode_ee as u16 {
                ir_data = ir_data
                    + params.il_chess_c[2] * (2 * il_pattern - 1) as f32
                    - params.il_chess_c[1] * conversion_pattern as f32;
            }

            ir_data -= params.tgc * ir_data_cp[subpage as usize];
            ir_data /= emissivity;

            let mut alpha_compensated =
                SCALE_ALPHA * alpha_scale / params.alpha[pixel] as f32;
            alpha_compensated *= 1.0 + params.ks_ta * (ta - 25.0);

            let ac3 = alpha_compensated * alpha_compensated * alpha_compensated;
            let sx = math::sqrt(math::sqrt(ac3 * (ir_data + alpha_compensated * ta_tr))) * params.ks_to[1];

            let mut to = math::sqrt(math::sqrt(
                ir_data / (alpha_compensated * (1.0 - params.ks_to[1] * 273.15) + sx) + ta_tr,
            )) - 273.15;

            let range: usize = if to < params.ct[1] as f32 {
                0
            } else if to < params.ct[2] as f32 {
                1
            } else if to < params.ct[3] as f32 {
                2
            } else {
                3
            };

            to = math::sqrt(math::sqrt(
                ir_data
                    / (alpha_compensated
                        * alpha_corr_r[range]
                        * (1.0 + params.ks_to[range] * (to - params.ct[range] as f32)))
                    + ta_tr,
            )) - 273.15;

            result[pixel] = to;
        }
    }

    ta
}
