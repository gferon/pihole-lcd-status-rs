use rppal::i2c::I2c;
use rppal::gpio::{ Mode, Level, PullUpDown };

use crate::errors::CommunicationError;

pub struct MCP230xx {
    device: I2c,
    num_gpio: u8,
    iodir: Vec<u8>,
    gppu: Vec<u8>,
    gpio: Vec<u8>,
}

const IODIR: u8 = 0x00;
const GPIO: u8 = 0x12;
const GPPU: u8 = 0x0C;

pub enum Pin {
    Up,
    Down,
}

/// MCP230xx series GPIO extender
impl MCP230xx {
    fn new(address: u8, num_gpio: u8) -> Result<Self, CommunicationError> {
        // Assume starting in ICON.BANK = 0 mode (sequential access).
        // Compute how many bytes are needed to store count of GPIO.
        let gpio_bytes = (num_gpio as f32 / 8.0).ceil() as usize;

        // Buffer register values so they can be changed without reading.
        let mut iodir = vec![];
        let mut gpio = vec![];
        let mut gppu = vec![];
        for _ in 0..gpio_bytes {
            iodir.push(0xFF);
            gpio.push(0x00);
            gppu.push(0x00);
        }

        let mut i2c_device = I2c::new().unwrap();
        i2c_device
            .set_slave_address(address as u16)
            .map_err(|e| return CommunicationError::BusError(e))?;

        let device = Self {
            device: i2c_device,
            num_gpio,
            iodir,
            gpio,
            gppu,
        };

        // Write current direction and pullup buffer state.
        device.write_iodir()?;
        device.write_gppu()?;
        Ok(device)
    }

    /// MCP23017-based GPIO class with 16 GPIO pins.
    pub fn for_mcp23017() -> Result<Self, CommunicationError> {
        Self::new(0x20, 16)
    }

    /// Checks that a pin is addressable, e.g. that the index is lower
    /// than the total of available GPIO ports.
    fn validate_pin(&self, pin: u8) -> Result<u8, CommunicationError> {
        if pin >= self.num_gpio {
            Err(CommunicationError::WrongPin(pin))
        } else {
            Ok(pin)
        }
    }

    /// Initialize a GPIO port as OUT or IN.
    pub fn setup(&mut self, pin: u8, mode: Mode) -> Result<(), CommunicationError> {
        self.validate_pin(pin)?;
        let idx = (pin / 8) as usize;
        match mode {
            Mode::Input => self.iodir[idx] |= 1 << (pin % 8),
            Mode::Output => self.iodir[idx] &= !(1 << (pin % 8)),
            _ => {}
        };
        self.write_iodir()
    }

    /// Turn on/off the pull-up resistor for the specified pin
    pub fn pullup(&mut self, pin: u8, pullupdown: PullUpDown) -> Result<(), CommunicationError> {
        self.validate_pin(pin)?;
        let idx = (pin / 8) as usize;
        match pullupdown {
            PullUpDown::PullUp => self.gppu[idx] |= 1 << (pin % 8),
            PullUpDown::PullDown => panic!("Unsupported"),
            PullUpDown::Off => self.gppu[idx] &= !(1 << (pin % 8)),
        }
        self.write_gppu()
    }

    /// Write the specified byte value to the IODIR registor.
    /// If no value specified the current buffered value will be written.
    fn write_iodir(&self) -> Result<(), CommunicationError> {
        self.device
            .block_write(IODIR, &self.iodir)
            .map_err(|e| CommunicationError::BusError(e))
    }

    fn write_gppu(&self) -> Result<(), CommunicationError> {
        self.device
            .block_write(GPPU, &self.gppu)
            .map_err(|e| CommunicationError::BusError(e))
    }

    fn write_gpio(&self) -> Result<(), CommunicationError> {
        self.device
            .block_write(GPIO, &self.gpio)
            .map_err(|e| CommunicationError::BusError(e))
    }

    pub fn output(&mut self, pin: u8, value: bool) -> Result<(), CommunicationError> {
        self.output_pins(&[(pin, value)])
    }

    pub fn output_pins(&mut self, pins: &[(u8, bool)]) -> Result<(), CommunicationError> {
        for (pin, value) in pins {
            self.validate_pin(*pin)?;
            let idx = (*pin / 8) as usize;
            let val = if *value {
                self.gpio[idx] | 1 << (pin % 8)
            } else {
                self.gpio[idx] & !(1 << (pin % 8))
            };
            self.gpio[idx] = val;
        }
        self.write_gpio()
    }

    /// Read multiple pins specified in the given list and return list of pin values
    pub fn input_pins(&mut self, pins: &[u8]) -> Result<Vec<Level>, CommunicationError> {
        pins.iter().map(|p| self.validate_pin(*p));
        self.device
            .block_read(GPIO, &mut self.gpio)
            .map_err(|e| CommunicationError::BusError(e))?;

        Ok(pins
            .iter()
            .map(|pin| {
                if self.gpio[(pin / 8) as usize] & 1 << (pin % 8) > 0 {
                    Level::High
                } else {
                    Level::Low
                }
            }).collect())
    }

    /// Read the specified pin and return its level
    pub fn input(&mut self, pin: u8) -> Result<Level, CommunicationError> {
        Ok(self.input_pins(&[pin])?[0])
    }
}
