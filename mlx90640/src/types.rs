use embedded_hal::i2c;

pub enum Error<I2C>
where
    I2C: i2c::I2c,
{
    I2cError(I2C::Error),
    TooManyBrokenPixels,
    TooManyOutlierPixels,
    TooManyBadPixels,
    AdjacentBadPixels,
    FrameDataError,
    Timeout,
}

impl<I2C: i2c::I2c> core::fmt::Debug for Error<I2C>
where
    I2C::Error: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::I2cError(e) => f.debug_tuple("I2cError").field(e).finish(),
            Error::TooManyBrokenPixels => f.write_str("TooManyBrokenPixels"),
            Error::TooManyOutlierPixels => f.write_str("TooManyOutlierPixels"),
            Error::TooManyBadPixels => f.write_str("TooManyBadPixels"),
            Error::AdjacentBadPixels => f.write_str("AdjacentBadPixels"),
            Error::FrameDataError => f.write_str("FrameDataError"),
            Error::Timeout => f.write_str("Timeout"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameRate {
    Half = 0,
    One = 1,
    Two = 2,
    Four = 3,
    Eight = 4,
    Sixteen = 5,
    ThirtyTwo = 6,
    SixtyFour = 7,
}

impl FrameRate {
    pub fn as_raw(self) -> u16 {
        match self {
            FrameRate::Half => 0,
            FrameRate::One => 1,
            FrameRate::Two => 2,
            FrameRate::Four => 3,
            FrameRate::Eight => 4,
            FrameRate::Sixteen => 5,
            FrameRate::ThirtyTwo => 6,
            FrameRate::SixtyFour => 7,
        }
    }

    pub fn from_raw(raw: u16) -> Option<Self> {
        match raw {
            0 => Some(FrameRate::Half),
            1 => Some(FrameRate::One),
            2 => Some(FrameRate::Two),
            3 => Some(FrameRate::Four),
            4 => Some(FrameRate::Eight),
            5 => Some(FrameRate::Sixteen),
            6 => Some(FrameRate::ThirtyTwo),
            7 => Some(FrameRate::SixtyFour),
            _ => None,
        }
    }
}

impl From<FrameRate> for f32 {
    fn from(fr: FrameRate) -> f32 {
        match fr {
            FrameRate::Half => 0.5,
            FrameRate::One => 1.0,
            FrameRate::Two => 2.0,
            FrameRate::Four => 4.0,
            FrameRate::Eight => 8.0,
            FrameRate::Sixteen => 16.0,
            FrameRate::ThirtyTwo => 32.0,
            FrameRate::SixtyFour => 64.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_rate_round_trip() {
        let rates = [
            FrameRate::Half,
            FrameRate::One,
            FrameRate::Two,
            FrameRate::Four,
            FrameRate::Eight,
            FrameRate::Sixteen,
            FrameRate::ThirtyTwo,
            FrameRate::SixtyFour,
        ];
        for &rate in &rates {
            assert_eq!(FrameRate::from_raw(rate.as_raw()), Some(rate));
        }
    }

    #[test]
    fn frame_rate_out_of_range() {
        assert_eq!(FrameRate::from_raw(8), None);
        assert_eq!(FrameRate::from_raw(255), None);
    }

    #[test]
    fn frame_rate_to_f32() {
        assert_eq!(f32::from(FrameRate::Half), 0.5);
        assert_eq!(f32::from(FrameRate::One), 1.0);
        assert_eq!(f32::from(FrameRate::Two), 2.0);
        assert_eq!(f32::from(FrameRate::Four), 4.0);
        assert_eq!(f32::from(FrameRate::Eight), 8.0);
        assert_eq!(f32::from(FrameRate::Sixteen), 16.0);
        assert_eq!(f32::from(FrameRate::ThirtyTwo), 32.0);
        assert_eq!(f32::from(FrameRate::SixtyFour), 64.0);
    }
}
