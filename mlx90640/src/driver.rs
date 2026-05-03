use embedded_hal::i2c::I2c;

use crate::bad_pixels;
use crate::calculations;
use crate::calibration::{self, CalibrationParams};
use crate::types::{Error, FrameRate};

const INIT_STATUS: u16 = 0x0030;
const MAX_WAIT_ITERATIONS: u32 = 2000;

pub struct Mlx90640<I2C> {
    i2c: I2C,
    params: CalibrationParams,
    ambient_temp: f32,
    emissivity: f32,
    tr: f32,
}

impl<I2C: I2c> Mlx90640<I2C> {
    pub fn new(i2c: I2C) -> Result<Self, Error<I2C>> {
        let mut i2c = i2c;
        let ee = calibration::read_ee(&mut i2c, crate::ADDRESS)?;
        let params = calibration::extract_parameters(&ee)?;

        Ok(Mlx90640 {
            i2c,
            params,
            ambient_temp: 25.0,
            emissivity: 0.95,
            tr: 25.0,
        })
    }

    pub fn set_frame_rate(&mut self, rate: FrameRate) -> Result<(), Error<I2C>> {
        calibration::set_frame_rate(&mut self.i2c, crate::ADDRESS, u16::from(rate))
    }

    pub fn set_emissivity(&mut self, e: f32) {
        self.emissivity = e;
    }

    pub fn set_tr(&mut self, tr: f32) {
        self.tr = tr;
    }

    pub fn ambient_temperature(&self) -> f32 {
        self.ambient_temp
    }

    pub fn generate_image(&mut self, dest: &mut [f32; 768]) -> Result<(), Error<I2C>> {
        for _ in 0..MAX_WAIT_ITERATIONS {
            let status = calibration::read_status(&mut self.i2c, crate::ADDRESS)?;
            if status & 0x0008 != 0 {
                break;
            }
        }

        let final_status = calibration::read_status(&mut self.i2c, crate::ADDRESS)?;
        if final_status & 0x0008 == 0 {
            return Err(Error::Timeout);
        }

        calibration::write_status(&mut self.i2c, crate::ADDRESS, INIT_STATUS)?;

        let mut frame_data = [0u16; 834];
        let pixel_data = calibration::read_pixel_ram(&mut self.i2c, crate::ADDRESS)?;
        frame_data[..pixel_data.len()].copy_from_slice(&pixel_data);

        let aux_data = calibration::read_aux_ram(&mut self.i2c, crate::ADDRESS)?;
        calibration::validate_aux_data(&aux_data)?;
        for (i, &val) in aux_data.iter().enumerate() {
            frame_data[pixel_data.len() + i] = val;
        }

        let ctrl = calibration::read_ctrl(&mut self.i2c, crate::ADDRESS)?;
        frame_data[832] = ctrl;
        let status = calibration::read_status(&mut self.i2c, crate::ADDRESS)?;
        frame_data[833] = status & 0x0001;

        calibration::validate_frame_data(&frame_data, frame_data[833])?;

        let ta = calculations::calculate_to(&frame_data, &self.params, self.emissivity, self.tr, dest);
        self.ambient_temp = ta;

        let mode = (ctrl >> 12) & 0x01;
        bad_pixels::correct_bad_pixels(&self.params.broken_pixels, dest, mode, &self.params);
        bad_pixels::correct_bad_pixels(&self.params.outlier_pixels, dest, mode, &self.params);

        Ok(())
    }
}
