use bmp::{self, Pixel};
use rppal::i2c::I2c;
use std::path::PathBuf;

use crate::errors::CommunicationError;

const HT16K33_BLINK_CMD: u8 = 0x80;
const HT16K33_SYSTEM_SETUP: u8 = 0x20;
const HT16K33_CMD_BRIGHTNESS: u8 = 0xE0;

pub trait BicolorMatrix8x8 {
    fn new(brightness: u8, blink: Blink) -> Result<Self, CommunicationError>
    where
        Self: std::marker::Sized;

    fn blink(&self, blink: Blink) -> Result<&Self, CommunicationError>;

    fn brightness(&self, brightness: u8) -> Result<&Self, CommunicationError>;

    fn set_pixel(
        &mut self,
        x: u8,
        y: u8,
        color: Color,
        write_display: bool,
    ) -> Result<(), CommunicationError>;

    fn set_image(&mut self, filepath: PathBuf) -> Result<(), CommunicationError>;

    fn clear(&mut self) -> Result<(), CommunicationError>;
}

pub struct HT16K33 {
    device: I2c,
    buffer: [u8; 16],
}

pub enum Blink {
    Off = 0x00,
    HalfHz = 0x06,
    TwoHz = 0x04,
    OneHz = 0x02,
}

#[derive(PartialEq, Clone)]
pub enum Color {
    Off = 0x00,
    Green = 0x01,
    Red = 0x02,
    Yellow = 0x03,
}

impl From<Pixel> for Color {
    fn from(pixel: Pixel) -> Self {
        match pixel {
            Pixel { r: 255, g: 0, b: 0 } => Color::Red,
            Pixel {
                r: 255,
                g: 255,
                b: 0,
            } => Color::Yellow,
            Pixel { r: 0, g: 255, b: 0 } => Color::Green,
            _ => Color::Off,
        }
    }
}

pub enum Bit {
    On = 0x01,
    Off = 0x00,
}

impl HT16K33 {
    /// The system setup register configures system operation or standby
    /// * The internal system oscillator is enabled when the 'S' bit of the system setup register is set to "1".
    /// * The internal system clock is disabled and the device will enter the standby mode when the "S" bit
    ///   of the system setup register is set to "0"
    pub fn system_setup(&self, operation: Bit) -> Result<(), CommunicationError> {
        self.device
            .smbus_send_byte(HT16K33_SYSTEM_SETUP | operation as u8)
            .map_err(|e| CommunicationError::BusError(e))
    }

    /// The display setup register configures the LED display on/off and the blinking frequency for the HT16K33.
    /// * The LED display is enabled when the 'D' bit of the display setup register is set to "1".
    /// * The LED display is disabled when the 'D' bit of the display setup register is set to "0".
    pub fn display_setup(&self, display: Bit, frequency: Blink) -> Result<(), CommunicationError> {
        self.device
            .block_write(HT16K33_BLINK_CMD | display as u8 | frequency as u8, &[])
            .map_err(|e| CommunicationError::BusError(e))?;
        Ok(())
    }

    pub fn set_led(&mut self, led: u8, value: u8, update: bool) -> Result<(), CommunicationError> {
        if led > 127 {
            panic!("LED must be between 0 and 127");
        }
        let pos: usize = led as usize / 8;
        let offset = led % 8;
        if value == 0 {
            self.buffer[pos] &= !(1 << offset)
        } else {
            self.buffer[pos] |= 1 << offset
        }
        if update {
            self.device
                .block_write(pos as u8, &[self.buffer[pos]])
                .map_err(|e| CommunicationError::BusError(e))?;
        }
        Ok(())
    }

    pub fn write_display(&self) -> Result<(), CommunicationError> {
        for (i, value) in self.buffer.iter().enumerate() {
            self.device
                .block_write(i as u8, &[*value])
                .map_err(|e| CommunicationError::BusError(e))?;
        }
        Ok(())
    }
}

impl BicolorMatrix8x8 for HT16K33 {
    fn new(brightness: u8, blink: Blink) -> Result<Self, CommunicationError> {
        let mut device = I2c::new().unwrap();
        device
            .set_slave_address(0x70 as u16)
            .map_err(|e| return CommunicationError::BusError(e))?;

        let driver = HT16K33 {
            device,
            buffer: [0; 16],
        };
        driver.system_setup(Bit::On)?;
        driver.brightness(brightness)?;
        driver.blink(blink)?;

        Ok(driver)
    }

    fn blink(&self, frequency: Blink) -> Result<&Self, CommunicationError> {
        self.display_setup(Bit::On, frequency)?;
        Ok(self)
    }

    fn brightness(&self, brightness: u8) -> Result<&Self, CommunicationError> {
        if brightness >= 16 {
            panic!("Brightness can't be more than 15");
        }
        self.device
            .block_write(HT16K33_CMD_BRIGHTNESS | brightness, &[])
            .map_err(|e| CommunicationError::BusError(e))?;
        Ok(self)
    }

    fn set_pixel(
        &mut self,
        x: u8,
        y: u8,
        color: Color,
        write_display: bool,
    ) -> Result<(), CommunicationError> {
        assert_eq!(x < 8, true);
        assert_eq!(y < 8, true);
        let (led1, led2) = match color {
            Color::Green => (1, 0),
            Color::Red => (0, 1),
            Color::Yellow => (1, 1),
            Color::Off => (0, 0),
        };
        self.set_led(y * 16 + x, led1, write_display)?;
        self.set_led(y * 16 + x + 8, led2, write_display)?;
        Ok(())
    }

    fn set_image(&mut self, filepath: PathBuf) -> Result<(), CommunicationError> {
        let img = bmp::open(filepath).map_err(|e| CommunicationError::BitmapError(e))?;
        if img.get_height() != 8 || img.get_width() != 8 {
            panic!("You need to provide a 8x8 BMP sprite");
        }

        for (x, y) in img.coordinates() {
            self.set_pixel(x as u8, y as u8, img.get_pixel(x, y).into(), false)?;
        }

        self.write_display()
    }

    fn clear(&mut self) -> Result<(), CommunicationError> {
        for i in 0..127 {
            self.set_led(i, 0, false)?;
        }
        self.write_display()
    }
}
