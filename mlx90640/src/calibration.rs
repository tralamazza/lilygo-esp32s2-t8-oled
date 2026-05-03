use crate::types::Error;
use embedded_hal::i2c::I2c;

pub(crate) const EEPROM_START: u16 = 0x2400;
const EEPROM_LEN: u16 = 832;
const PIXEL_DATA_START: u16 = 0x0400;
const PIXEL_NUM: u16 = 768;
const AUX_DATA_START: u16 = 0x0700;
const AUX_NUM: u16 = 64;

const STATUS_REG: u16 = 0x8000;
const CTRL_REG: u16 = 0x800D;

const CTRL_REFRESH_SHIFT: u16 = 7;
const CTRL_REFRESH_MASK: u16 = !(0b111 << CTRL_REFRESH_SHIFT);

const LINE_NUM: usize = 24;
const COLUMN_NUM: usize = 32;
const LINE_SIZE: usize = 32;

pub(crate) const SCALE_ALPHA: f32 = 0.000001;

const NIBBLE1_MASK: u16 = 0x000F;
const NIBBLE2_MASK: u16 = 0x00F0;
const NIBBLE3_MASK: u16 = 0x0F00;
const NIBBLE4_MASK: u16 = 0xF000;
const MSBITS_6_MASK: u16 = 0xFC00;
const LSBITS_10_MASK: u16 = 0x03FF;
const MS_BYTE_MASK: u16 = 0xFF00;
const LS_BYTE_MASK: u16 = 0x00FF;

#[derive(Clone, Debug)]
pub(crate) struct CalibrationParams {
    pub kvdd: i16,
    pub vdd25: i16,
    pub kvptat: f32,
    pub ktptat: f32,
    pub vptat25: u16,
    pub alpha_ptat: f32,
    pub gain_ee: i16,
    pub tgc: f32,
    pub resolution_ee: u8,
    pub calibration_mode_ee: u8,
    pub ks_ta: f32,
    pub ks_to: [f32; 5],
    pub ct: [i16; 5],
    pub alpha: [u16; 768],
    pub alpha_scale: u8,
    pub offset: [i16; 768],
    pub kta: [i8; 768],
    pub kta_scale: u8,
    pub kv: [i8; 768],
    pub kv_scale: u8,
    pub cp_alpha: [f32; 2],
    pub cp_offset: [i16; 2],
    pub cp_kta: f32,
    pub cp_kv: f32,
    pub il_chess_c: [f32; 3],
    pub broken_pixels: [u16; 5],
    pub outlier_pixels: [u16; 5],
}

fn nibble1(w: u16) -> u16 { w & NIBBLE1_MASK }
fn nibble2(w: u16) -> u16 { (w & NIBBLE2_MASK) >> 4 }
fn nibble3(w: u16) -> u16 { (w & NIBBLE3_MASK) >> 8 }
fn nibble4(w: u16) -> u16 { (w & NIBBLE4_MASK) >> 12 }
fn ms_byte(w: u16) -> u8 { ((w & MS_BYTE_MASK) >> 8) as u8 }
fn ls_byte(w: u16) -> u8 { (w & LS_BYTE_MASK) as u8 }
fn msbits6(w: u16) -> u16 { (w & MSBITS_6_MASK) >> 10 }
fn lsbits10(w: u16) -> u16 { w & LSBITS_10_MASK }

pub(crate) fn read_ee<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
) -> Result<[u16; EEPROM_LEN as usize], Error<I2C>> {
    let mut buf = [0u8; EEPROM_LEN as usize * 2];
    i2c.write_read(addr, &EEPROM_START.to_be_bytes(), &mut buf)
        .map_err(Error::I2cError)?;
    let mut words = [0u16; EEPROM_LEN as usize];
    for (i, chunk) in buf.chunks_exact(2).enumerate() {
        words[i] = u16::from_be_bytes([chunk[0], chunk[1]]);
    }
    Ok(words)
}

pub(crate) fn read_pixel_ram<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
) -> Result<[u16; PIXEL_NUM as usize], Error<I2C>> {
    let mut buf = [0u8; PIXEL_NUM as usize * 2];
    i2c.write_read(addr, &PIXEL_DATA_START.to_be_bytes(), &mut buf)
        .map_err(Error::I2cError)?;
    let mut words = [0u16; PIXEL_NUM as usize];
    for (i, chunk) in buf.chunks_exact(2).enumerate() {
        words[i] = u16::from_be_bytes([chunk[0], chunk[1]]);
    }
    Ok(words)
}

pub(crate) fn read_aux_ram<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
) -> Result<[u16; AUX_NUM as usize], Error<I2C>> {
    let mut buf = [0u8; AUX_NUM as usize * 2];
    i2c.write_read(addr, &AUX_DATA_START.to_be_bytes(), &mut buf)
        .map_err(Error::I2cError)?;
    let mut words = [0u16; AUX_NUM as usize];
    for (i, chunk) in buf.chunks_exact(2).enumerate() {
        words[i] = u16::from_be_bytes([chunk[0], chunk[1]]);
    }
    Ok(words)
}

pub(crate) fn read_register<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
    reg: u16,
) -> Result<u16, Error<I2C>> {
    let mut buf = [0u8; 2];
    i2c.write_read(addr, &reg.to_be_bytes(), &mut buf)
        .map_err(Error::I2cError)?;
    Ok(u16::from_be_bytes([buf[0], buf[1]]))
}

pub(crate) fn write_register<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
    reg: u16,
    value: u16,
) -> Result<(), Error<I2C>> {
    let reg_bytes = reg.to_be_bytes();
    let val_bytes = value.to_be_bytes();
    i2c.write(addr, &[reg_bytes[0], reg_bytes[1], val_bytes[0], val_bytes[1]])
        .map_err(Error::I2cError)
}

pub(crate) fn validate_frame_data<I2C: I2c>(frame_data: &[u16], subpage: u16) -> Result<(), Error<I2C>> {
    for (line, i) in (0..).zip((0..PIXEL_NUM as usize).step_by(LINE_SIZE)) {
        if frame_data[i] == 0x7FFF && (line % 2) as u16 == subpage {
            return Err(Error::FrameDataError);
        }
    }
    Ok(())
}

pub(crate) fn validate_aux_data<I2C: I2c>(aux_data: &[u16]) -> Result<(), Error<I2C>> {
    if aux_data[0] == 0x7FFF {
        return Err(Error::FrameDataError);
    }
    for &range in &[(8, 19), (20, 23), (24, 33), (40, 51), (52, 55), (56, 64)] {
        for &val in &aux_data[range.0..range.1] {
            if val == 0x7FFF {
                return Err(Error::FrameDataError);
            }
        }
    }
    Ok(())
}

fn pow2(exp: i32) -> f32 {
    crate::math::pow(2.0, exp as f32)
}

pub(crate) fn extract_parameters<I2C: I2c>(ee_data: &[u16; EEPROM_LEN as usize]) -> Result<CalibrationParams, Error<I2C>> {
    let mut p = CalibrationParams {
        kvdd: 0,
        vdd25: 0,
        kvptat: 0.0,
        ktptat: 0.0,
        vptat25: 0,
        alpha_ptat: 0.0,
        gain_ee: 0,
        tgc: 0.0,
        resolution_ee: 0,
        calibration_mode_ee: 0,
        ks_ta: 0.0,
        ks_to: [0.0; 5],
        ct: [0; 5],
        alpha: [0; 768],
        alpha_scale: 0,
        offset: [0; 768],
        kta: [0; 768],
        kta_scale: 0,
        kv: [0; 768],
        kv_scale: 0,
        cp_alpha: [0.0; 2],
        cp_offset: [0; 2],
        cp_kta: 0.0,
        cp_kv: 0.0,
        il_chess_c: [0.0; 3],
        broken_pixels: [0xFFFF; 5],
        outlier_pixels: [0xFFFF; 5],
    };

    // VDD params
    {
        let kvdd = ms_byte(ee_data[51]) as i8;
        let mut vdd25 = ls_byte(ee_data[51]) as i16 as i32;
        vdd25 = ((vdd25 - 256) << 5) - 8192;
        p.kvdd = 32 * kvdd as i16;
        p.vdd25 = vdd25 as i16;
    }

    // PTAT params
    {
        let mut kvptat = ((ee_data[50] & MSBITS_6_MASK) >> 10) as i32;
        if kvptat > 31 { kvptat -= 64; }
        p.kvptat = kvptat as f32 / 4096.0;

        let mut ktptat = (ee_data[50] & LSBITS_10_MASK) as i32;
        if ktptat > 511 { ktptat -= 1024; }
        p.ktptat = ktptat as f32 / 8.0;

        p.vptat25 = ee_data[49];
        p.alpha_ptat = (ee_data[16] & NIBBLE4_MASK) as f32 / pow2(14) + 8.0;
    }

    // Gain
    p.gain_ee = ee_data[48] as i16;

    // TGC
    p.tgc = ls_byte(ee_data[60]) as i8 as f32 / 32.0;

    // Resolution
    p.resolution_ee = ((ee_data[56] & 0x3000) >> 12) as u8;

    // KsTa
    p.ks_ta = ms_byte(ee_data[60]) as i8 as f32 / 8192.0;

    // KsTo + corner temps
    {
        let step = ((ee_data[63] & 0x3000) >> 12) * 10;
        p.ct[0] = -40;
        p.ct[1] = 0;
        p.ct[2] = nibble2(ee_data[63]) as i16;
        p.ct[3] = nibble3(ee_data[63]) as i16;
        p.ct[2] *= step as i16;
        p.ct[3] = p.ct[2] + p.ct[3] * step as i16;
        p.ct[4] = 400;

        let ks_to_scale: u32 = (nibble1(ee_data[63]) + 8) as u32;
        let ks_to_scale_f = (1u32 << ks_to_scale) as f32;

        p.ks_to[0] = ls_byte(ee_data[61]) as i8 as f32 / ks_to_scale_f;
        p.ks_to[1] = ms_byte(ee_data[61]) as i8 as f32 / ks_to_scale_f;
        p.ks_to[2] = ls_byte(ee_data[62]) as i8 as f32 / ks_to_scale_f;
        p.ks_to[3] = ms_byte(ee_data[62]) as i8 as f32 / ks_to_scale_f;
        p.ks_to[4] = -0.0002;
    }

    // CP params
    {
        let alpha_scale = nibble4(ee_data[32]) as i32 + 27;

        let mut offset_sp0 = lsbits10(ee_data[58]) as i32;
        if offset_sp0 > 511 { offset_sp0 -= 1024; }

        let mut offset_sp1 = msbits6(ee_data[58]) as i32;
        if offset_sp1 > 31 { offset_sp1 -= 64; }
        offset_sp1 += offset_sp0;

        p.cp_offset[0] = offset_sp0 as i16;
        p.cp_offset[1] = offset_sp1 as i16;

        let mut alpha_sp0 = lsbits10(ee_data[57]) as i32;
        if alpha_sp0 > 511 { alpha_sp0 -= 1024; }
        p.cp_alpha[0] = alpha_sp0 as f32 / pow2(alpha_scale);

        let mut alpha_sp1 = msbits6(ee_data[57]) as i32;
        if alpha_sp1 > 31 { alpha_sp1 -= 64; }
        p.cp_alpha[1] = (1.0 + alpha_sp1 as f32 / 128.0) * p.cp_alpha[0];

        let kta_scale1 = nibble2(ee_data[56]) as i32 + 8;
        p.cp_kta = ls_byte(ee_data[59]) as i8 as f32 / pow2(kta_scale1);

        let kv_scale = nibble3(ee_data[56]);
        p.cp_kv = ms_byte(ee_data[59]) as i8 as f32 / pow2(kv_scale as i32);
    }

    // CILC params (interleave/chess corrections + calibration mode)
    {
        p.calibration_mode_ee = ((ee_data[10] & 0x0800) >> 4) as u8;
        p.calibration_mode_ee ^= 0x80;

        let mut ilc0 = (ee_data[53] & 0x003F) as i32;
        if ilc0 > 31 { ilc0 -= 64; }
        p.il_chess_c[0] = ilc0 as f32 / 16.0;

        let mut ilc1 = ((ee_data[53] & 0x07C0) >> 6) as i32;
        if ilc1 > 15 { ilc1 -= 32; }
        p.il_chess_c[1] = ilc1 as f32 / 2.0;

        let mut ilc2 = ((ee_data[53] & 0xF800) >> 11) as i32;
        if ilc2 > 15 { ilc2 -= 32; }
        p.il_chess_c[2] = ilc2 as f32 / 8.0;
    }

    // Alpha params
    {
        let acc_rem_scale = nibble1(ee_data[32]);
        let acc_column_scale = nibble2(ee_data[32]);
        let acc_row_scale = nibble3(ee_data[32]);
        let alpha_scale_exp = nibble4(ee_data[32]) as i32 + 30;
        let alpha_ref = ee_data[33] as i32;

        let mut acc_row = [0i32; 24];
        for i in 0..6 {
            let w = ee_data[34 + i];
            acc_row[i * 4] = nibble1(w) as i32;
            acc_row[i * 4 + 1] = nibble2(w) as i32;
            acc_row[i * 4 + 2] = nibble3(w) as i32;
            acc_row[i * 4 + 3] = nibble4(w) as i32;
        }
        for r in acc_row.iter_mut() {
            if *r > 7 { *r -= 16; }
        }

        let mut acc_column = [0i32; 32];
        for i in 0..8 {
            let w = ee_data[40 + i];
            acc_column[i * 4] = nibble1(w) as i32;
            acc_column[i * 4 + 1] = nibble2(w) as i32;
            acc_column[i * 4 + 2] = nibble3(w) as i32;
            acc_column[i * 4 + 3] = nibble4(w) as i32;
        }
        for c in acc_column.iter_mut() {
            if *c > 7 { *c -= 16; }
        }

        let mut alpha_temp = [0.0f32; 768];
        for (i, row) in acc_row.iter().enumerate().take(LINE_NUM) {
            for (j, col) in acc_column.iter().enumerate().take(COLUMN_NUM) {
                let pix = 32 * i + j;
                let mut a = ((ee_data[64 + pix] & 0x03F0) >> 4) as i32;
                if a > 31 { a -= 64; }
                let mut val = a << acc_rem_scale;
                val += alpha_ref + (row << acc_row_scale) + (col << acc_column_scale);
                alpha_temp[pix] = val as f32 / pow2(alpha_scale_exp);
                alpha_temp[pix] -= p.tgc * (p.cp_alpha[0] + p.cp_alpha[1]) / 2.0;
                alpha_temp[pix] = SCALE_ALPHA / alpha_temp[pix];
            }
        }

        let mut max_val = alpha_temp[0];
        for &v in alpha_temp.iter().skip(1) {
            if v > max_val { max_val = v; }
        }

        let mut a_scale: u8 = 0;
        let mut temp = max_val;
        while temp < 32767.4 {
            temp *= 2.0;
            a_scale += 1;
        }

        for (i, &at) in alpha_temp.iter().enumerate() {
            let val = at * pow2(a_scale as i32);
            if val < 0.0 {
                p.alpha[i] = (val - 0.5) as i32 as u16;
            } else {
                p.alpha[i] = (val + 0.5) as u16;
            }
        }
        p.alpha_scale = a_scale;
    }

    // Offset params
    {
        let occ_rem_scale = nibble1(ee_data[16]);
        let occ_column_scale = nibble2(ee_data[16]);
        let occ_row_scale = nibble3(ee_data[16]);
        let offset_ref = ee_data[17] as i32;

        let mut occ_row = [0i32; 24];
        for i in 0..6 {
            let w = ee_data[18 + i];
            occ_row[i * 4] = nibble1(w) as i32;
            occ_row[i * 4 + 1] = nibble2(w) as i32;
            occ_row[i * 4 + 2] = nibble3(w) as i32;
            occ_row[i * 4 + 3] = nibble4(w) as i32;
        }
        for r in occ_row.iter_mut() {
            if *r > 7 { *r -= 16; }
        }

        let mut occ_column = [0i32; 32];
        for i in 0..8 {
            let w = ee_data[24 + i];
            occ_column[i * 4] = nibble1(w) as i32;
            occ_column[i * 4 + 1] = nibble2(w) as i32;
            occ_column[i * 4 + 2] = nibble3(w) as i32;
            occ_column[i * 4 + 3] = nibble4(w) as i32;
        }
        for c in occ_column.iter_mut() {
            if *c > 7 { *c -= 16; }
        }

        for (i, row) in occ_row.iter().enumerate().take(LINE_NUM) {
            for (j, col) in occ_column.iter().enumerate().take(COLUMN_NUM) {
                let pix = 32 * i + j;
                let mut off = msbits6(ee_data[64 + pix]) as i32;
                if off > 31 { off -= 64; }
                off <<= occ_rem_scale;
                off += offset_ref + (row << occ_row_scale) + (col << occ_column_scale);
                p.offset[pix] = off as i16;
            }
        }
    }

    // Kta pixel params
    {
        let mut kta_rc = [0i8; 4];
        kta_rc[0] = ms_byte(ee_data[54]) as i8;
        kta_rc[2] = ls_byte(ee_data[54]) as i8;
        kta_rc[1] = ms_byte(ee_data[55]) as i8;
        kta_rc[3] = ls_byte(ee_data[55]) as i8;

        let kta_scale1 = nibble2(ee_data[56]) as i32 + 8;
        let kta_scale2 = nibble1(ee_data[56]);

        let mut kta_temp = [0.0f32; 768];
        for i in 0..LINE_NUM {
            for j in 0..COLUMN_NUM {
                let pix = 32 * i + j;
                let split = 2 * (pix as i32 / 32 - (pix as i32 / 64) * 2) + (pix as i32 % 2);
                let mut kt = ((ee_data[64 + pix] & 0x000E) >> 1) as i32;
                if kt > 3 { kt -= 8; }
                kt <<= kta_scale2;
                kt += kta_rc[split as usize] as i32;
                kta_temp[pix] = kt as f32 / pow2(kta_scale1);
            }
        }

        let mut max_val = crate::math::abs(kta_temp[0]);
        for &v in kta_temp.iter().skip(1) {
            let a = crate::math::abs(v);
            if a > max_val { max_val = a; }
        }

        let mut k_scale: u8 = 0;
        let mut temp_val = max_val;
        while temp_val < 63.4 {
            temp_val *= 2.0;
            k_scale += 1;
        }

        for (i, &kt) in kta_temp.iter().enumerate() {
            let val = kt * pow2(k_scale as i32);
            if val < 0.0 {
                p.kta[i] = (val - 0.5) as i8;
            } else {
                p.kta[i] = (val + 0.5) as i8;
            }
        }
        p.kta_scale = k_scale;
    }

    // Kv pixel params
    {
        let mut kv_t = [0i8; 4];

        let mut k = nibble4(ee_data[52]) as i32;
        if k > 7 { k -= 16; }
        kv_t[0] = k as i8;

        k = nibble3(ee_data[52]) as i32;
        if k > 7 { k -= 16; }
        kv_t[2] = k as i8;

        k = nibble2(ee_data[52]) as i32;
        if k > 7 { k -= 16; }
        kv_t[1] = k as i8;

        k = nibble1(ee_data[52]) as i32;
        if k > 7 { k -= 16; }
        kv_t[3] = k as i8;

        let kv_scale = nibble3(ee_data[56]);

        let mut kv_temp = [0.0f32; 768];
        for i in 0..LINE_NUM {
            for j in 0..COLUMN_NUM {
                let pix = 32 * i + j;
                let split = 2 * (pix / 32 - (pix / 64) * 2) + (pix % 2);
                kv_temp[pix] = kv_t[split] as f32 / pow2(kv_scale as i32);
            }
        }

        let mut max_val = crate::math::abs(kv_temp[0]);
        for &v in kv_temp.iter().skip(1) {
            let a = crate::math::abs(v);
            if a > max_val { max_val = a; }
        }

        let mut k_scale: u8 = 0;
        let mut temp_val = max_val;
        while temp_val < 63.4 {
            temp_val *= 2.0;
            k_scale += 1;
        }

        for (i, &kv) in kv_temp.iter().enumerate() {
            let val = kv * pow2(k_scale as i32);
            if val < 0.0 {
                p.kv[i] = (val - 0.5) as i8;
            } else {
                p.kv[i] = (val + 0.5) as i8;
            }
        }
        p.kv_scale = k_scale;
    }

    // Deviating pixels
    {
        let mut broken_cnt = 0;
        let mut outlier_cnt = 0;

        for pix in 0..PIXEL_NUM as usize {
            if ee_data[pix + 64] == 0 {
                if broken_cnt < 5 {
                    p.broken_pixels[broken_cnt] = pix as u16;
                    broken_cnt += 1;
                }
            } else if (ee_data[pix + 64] & 0x0001) != 0
                && outlier_cnt < 5 {
                    p.outlier_pixels[outlier_cnt] = pix as u16;
                    outlier_cnt += 1;
                }
        }

        if broken_cnt > 4 {
            return Err(Error::TooManyBrokenPixels);
        }
        if outlier_cnt > 4 {
            return Err(Error::TooManyOutlierPixels);
        }
        if (broken_cnt + outlier_cnt) > 4 {
            return Err(Error::TooManyBadPixels);
        }

        // Check adjacent broken pixels
        for i in 0..broken_cnt {
            for j in (i + 1)..broken_cnt {
                if check_adjacent_pixels(p.broken_pixels[i], p.broken_pixels[j]) {
                    return Err(Error::AdjacentBadPixels);
                }
            }
        }

        // Check adjacent outlier pixels
        for i in 0..outlier_cnt {
            for j in (i + 1)..outlier_cnt {
                if check_adjacent_pixels(p.outlier_pixels[i], p.outlier_pixels[j]) {
                    return Err(Error::AdjacentBadPixels);
                }
            }
        }

        // Check cross-adjacent
        for i in 0..broken_cnt {
            for j in 0..outlier_cnt {
                if check_adjacent_pixels(p.broken_pixels[i], p.outlier_pixels[j]) {
                    return Err(Error::AdjacentBadPixels);
                }
            }
        }
    }

    Ok(p)
}

fn check_adjacent_pixels(pix1: u16, pix2: u16) -> bool {
    let lp1 = pix1 >> 5;
    let lp2 = pix2 >> 5;
    let cp1 = pix1 - (lp1 << 5);
    let cp2 = pix2 - (lp2 << 5);

    let row_diff = (lp1 as i32) - (lp2 as i32);
    if row_diff > -2 && row_diff < 2 {
        let col_diff = (cp1 as i32) - (cp2 as i32);
        if col_diff > -2 && col_diff < 2 {
            return true;
        }
    }
    false
}

pub(crate) fn set_frame_rate<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
    rate_raw: u16,
) -> Result<(), Error<I2C>> {
    let ctrl = read_register(i2c, addr, CTRL_REG)?;
    let value = (ctrl & CTRL_REFRESH_MASK) | ((rate_raw << CTRL_REFRESH_SHIFT) & !CTRL_REFRESH_MASK);
    write_register(i2c, addr, CTRL_REG, value)
}

pub(crate) fn read_status<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
) -> Result<u16, Error<I2C>> {
    read_register(i2c, addr, STATUS_REG)
}

pub(crate) fn write_status<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
    value: u16,
) -> Result<(), Error<I2C>> {
    write_register(i2c, addr, STATUS_REG, value)
}

pub(crate) fn read_ctrl<I2C: I2c>(
    i2c: &mut I2C,
    addr: u8,
) -> Result<u16, Error<I2C>> {
    read_register(i2c, addr, CTRL_REG)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockI2c;

    impl embedded_hal::i2c::ErrorType for MockI2c {
        type Error = core::convert::Infallible;
    }

    impl embedded_hal::i2c::I2c for MockI2c {
        fn transaction(
            &mut self,
            _address: u8,
            _operations: &mut [embedded_hal::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    fn ok<T>(r: Result<T, Error<MockI2c>>) -> bool {
        r.is_ok()
    }

    #[test]
    fn nibble_extraction() {
        let w: u16 = 0xABCD;
        assert_eq!(nibble1(w), 0xD);
        assert_eq!(nibble2(w), 0xC);
        assert_eq!(nibble3(w), 0xB);
        assert_eq!(nibble4(w), 0xA);
    }

    #[test]
    fn nibble_mask_boundaries() {
        assert_eq!(nibble1(0xF), 0xF);
        assert_eq!(nibble1(0x10), 0x0);
        assert_eq!(nibble2(0xF0), 0xF);
        assert_eq!(nibble3(0xF00), 0xF);
        assert_eq!(nibble4(0xF000), 0xF);
    }

    #[test]
    fn ms_ls_byte() {
        let w: u16 = 0xABCD;
        assert_eq!(ms_byte(w), 0xAB);
        assert_eq!(ls_byte(w), 0xCD);
    }

    #[test]
    fn msbits6_lsbits10() {
        let w: u16 = 0xFC00 | 63;
        assert_eq!(msbits6(w), 63);
        assert_eq!(lsbits10(w), 63);

        let w2: u16 = (42 << 10) | 42;
        assert_eq!(msbits6(w2), 42);
        assert_eq!(lsbits10(w2), 42);
    }

    #[test]
    fn check_adjacent_same_pixel() {
        assert!(check_adjacent_pixels(0, 0));
    }

    #[test]
    fn check_adjacent_neighbors() {
        assert!(check_adjacent_pixels(0, 33));
        assert!(check_adjacent_pixels(33, 32));
        assert!(check_adjacent_pixels(32, 1));
    }

    #[test]
    fn check_adjacent_diagonal() {
        assert!(check_adjacent_pixels(0, 33));
        assert!(check_adjacent_pixels(33, 65));
    }

    #[test]
    fn check_adjacent_distant() {
        assert!(!check_adjacent_pixels(0, 100));
        assert!(!check_adjacent_pixels(0, 96));
    }

    #[test]
    fn check_adjacent_row_boundary() {
        assert!(!check_adjacent_pixels(0, 64));
    }

    #[test]
    fn validate_aux_data_good() {
        let aux = [0u16; 64];
        assert!(ok(validate_aux_data::<MockI2c>(&aux)));
    }

    #[test]
    fn validate_aux_data_bad_index_0() {
        let mut aux = [0u16; 64];
        aux[0] = 0x7FFF;
        assert!(!ok(validate_aux_data::<MockI2c>(&aux)));
    }

    #[test]
    fn validate_aux_data_bad_in_range() {
        let mut aux = [0u16; 64];
        aux[9] = 0x7FFF;
        assert!(!ok(validate_aux_data::<MockI2c>(&aux)));
    }

    #[test]
    fn validate_aux_data_sentinel_outside_range_ok() {
        let mut aux = [0u16; 64];
        aux[7] = 0x7FFF;
        assert!(ok(validate_aux_data::<MockI2c>(&aux)));
    }

    #[test]
    fn validate_frame_data_good() {
        let mut fd = [0u16; 834];
        for (i, v) in fd.iter_mut().enumerate().step_by(32) {
            *v = i as u16;
        }
        assert!(ok(validate_frame_data::<MockI2c>(&fd, 0)));
    }

    #[test]
    fn validate_frame_data_sentinel_on_wrong_subpage_ok() {
        let mut fd = [0u16; 834];
        fd[0] = 0x7FFF;
        assert!(ok(validate_frame_data::<MockI2c>(&fd, 1)));
    }

    #[test]
    fn validate_frame_data_sentinel_on_correct_subpage_err() {
        let mut fd = [0u16; 834];
        fd[0] = 0x7FFF;
        assert!(!ok(validate_frame_data::<MockI2c>(&fd, 0)));
    }

    #[test]
    fn validate_frame_data_sentinel_on_odd_line_even_subpage() {
        let mut fd = [0u16; 834];
        fd[32] = 0x7FFF;
        assert!(!ok(validate_frame_data::<MockI2c>(&fd, 1)));
        assert!(ok(validate_frame_data::<MockI2c>(&fd, 0)));
    }
}
