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

impl<I2C: i2c::I2c> Clone for Error<I2C>
where
    I2C::Error: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::I2cError(e) => Self::I2cError(e.clone()),
            Self::TooManyBrokenPixels => Self::TooManyBrokenPixels,
            Self::TooManyOutlierPixels => Self::TooManyOutlierPixels,
            Self::TooManyBadPixels => Self::TooManyBadPixels,
            Self::AdjacentBadPixels => Self::AdjacentBadPixels,
            Self::FrameDataError => Self::FrameDataError,
            Self::Timeout => Self::Timeout,
        }
    }
}

impl<I2C: i2c::I2c> PartialEq for Error<I2C>
where
    I2C::Error: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::I2cError(a), Self::I2cError(b)) => a == b,
            (Self::TooManyBrokenPixels, Self::TooManyBrokenPixels) => true,
            (Self::TooManyOutlierPixels, Self::TooManyOutlierPixels) => true,
            (Self::TooManyBadPixels, Self::TooManyBadPixels) => true,
            (Self::AdjacentBadPixels, Self::AdjacentBadPixels) => true,
            (Self::FrameDataError, Self::FrameDataError) => true,
            (Self::Timeout, Self::Timeout) => true,
            _ => false,
        }
    }
}

impl<I2C: i2c::I2c> core::fmt::Debug for Error<I2C>
where
    I2C::Error: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::I2cError(e) => f.debug_tuple("I2cError").field(e).finish(),
            Self::TooManyBrokenPixels => f.write_str("TooManyBrokenPixels"),
            Self::TooManyOutlierPixels => f.write_str("TooManyOutlierPixels"),
            Self::TooManyBadPixels => f.write_str("TooManyBadPixels"),
            Self::AdjacentBadPixels => f.write_str("AdjacentBadPixels"),
            Self::FrameDataError => f.write_str("FrameDataError"),
            Self::Timeout => f.write_str("Timeout"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
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

impl From<FrameRate> for u16 {
    fn from(fr: FrameRate) -> u16 {
        fr as u16
    }
}

impl TryFrom<u16> for FrameRate {
    type Error = ();

    fn try_from(raw: u16) -> Result<Self, Self::Error> {
        match raw {
            0 => Ok(Self::Half),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Four),
            4 => Ok(Self::Eight),
            5 => Ok(Self::Sixteen),
            6 => Ok(Self::ThirtyTwo),
            7 => Ok(Self::SixtyFour),
            _ => Err(()),
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
            assert_eq!(FrameRate::try_from(u16::from(rate)), Ok(rate));
        }
    }

    #[test]
    fn frame_rate_out_of_range() {
        assert_eq!(FrameRate::try_from(8u16), Err(()));
        assert_eq!(FrameRate::try_from(255u16), Err(()));
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
