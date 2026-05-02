use core::marker::PhantomData;
use embedded_hal::i2c::I2c;

use crate::bad_pixels;
use crate::calculations;
use crate::calibration::{self, CalibrationParams};
use crate::types::{Error, FrameRate};

const ADDR: u8 = 0x33;
const INIT_STATUS: u16 = 0x0030;

pub struct Mlx90640<I2C> {
    i2c: I2C,
    params: CalibrationParams,
    ambient_temp: f32,
    emissivity: f32,
    _marker: PhantomData<I2C>,
}

impl<I2C: I2c> Mlx90640<I2C> {
    pub fn new(mut i2c: I2C) -> Result<Self, Error<I2C>> {
        let ee = calibration::read_ee(&mut i2c, ADDR)?;
        let params = calibration::extract_parameters(&ee)?;

        let mut driver = Mlx90640 {
            i2c,
            params,
            ambient_temp: 25.0,
            emissivity: 0.95,
            _marker: PhantomData,
        };
        driver.set_frame_rate(FrameRate::Eight)?;
        Ok(driver)
    }

    pub fn set_frame_rate(&mut self, rate: FrameRate) -> Result<(), Error<I2C>> {
        calibration::set_frame_rate(&mut self.i2c, ADDR, rate.as_raw())
    }

    pub fn ambient_temperature(&self) -> f32 {
        self.ambient_temp
    }

    pub fn generate_image(&mut self, dest: &mut [f32; 768]) -> Result<(), Error<I2C>> {
        loop {
            let status = calibration::read_status(&mut self.i2c, ADDR)?;
            if status & 0x0008 != 0 {
                break;
            }
        }

        calibration::write_status(&mut self.i2c, ADDR, INIT_STATUS)?;

        let mut frame_data = [0u16; 834];
        let pixel_data = calibration::read_pixel_ram(&mut self.i2c, ADDR)?;
        frame_data[..pixel_data.len()].copy_from_slice(&pixel_data);

        let aux_data = calibration::read_aux_ram(&mut self.i2c, ADDR)?;
        calibration::validate_aux_data(&aux_data)?;
        for (i, &val) in aux_data.iter().enumerate() {
            frame_data[pixel_data.len() + i] = val;
        }

        let ctrl = calibration::read_ctrl(&mut self.i2c, ADDR)?;
        frame_data[832] = ctrl;
        let status = calibration::read_status(&mut self.i2c, ADDR)?;
        frame_data[833] = status & 0x0001;

        calibration::validate_frame_data(&frame_data, frame_data[833])?;

        let ta = calculations::calculate_to(&frame_data, &self.params, self.emissivity, 25.0, dest);
        self.ambient_temp = ta;

        let mode = (ctrl >> 12) & 0x01;
        bad_pixels::correct_bad_pixels(&self.params.broken_pixels, dest, mode, &self.params);
        bad_pixels::correct_bad_pixels(&self.params.outlier_pixels, dest, mode, &self.params);

        Ok(())
    }
}
