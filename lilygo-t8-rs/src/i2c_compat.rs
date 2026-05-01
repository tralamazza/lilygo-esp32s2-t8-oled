use embedded_hal::i2c::Operation as Op10;

pub struct I2cCompat<I2C> {
    inner: I2C,
}

impl<I2C> I2cCompat<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    pub fn new(inner: I2C) -> Self {
        Self { inner }
    }
}

impl<I2C> embedded_hal_02::blocking::i2c::Write for I2cCompat<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = I2C::Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.inner.transaction(addr, &mut [Op10::Write(bytes)])
    }
}

impl<I2C> embedded_hal_02::blocking::i2c::Read for I2cCompat<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = I2C::Error;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.transaction(addr, &mut [Op10::Read(buffer)])
    }
}

impl<I2C> embedded_hal_02::blocking::i2c::WriteRead for I2cCompat<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = I2C::Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.inner
            .transaction(addr, &mut [Op10::Write(bytes), Op10::Read(buffer)])
    }
}
