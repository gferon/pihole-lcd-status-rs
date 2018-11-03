use std::char;

pub mod helpers;

use crate::errors::CommunicationError;
use crate::i2c::io::MCP230xx;

use rppal::gpio::Mode;

// Commands
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const _LCD_CURSORSHIFT: u8 = 0x10;
const LCD_FUNCTIONSET: u8 = 0x20;
const LCD_SETCGRAMADDR: u8 = 0x40;
const LCD_SETDDRAMADDR: u8 = 0x80;

// Entry flags
const _LCD_ENTRYRIGHT: u8 = 0x00;
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// Control flags
const LCD_DISPLAYON: u8 = 0x04;
const _LCD_DISPLAYOFF: u8 = 0x00;
const _LCD_CURSORON: u8 = 0x02;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKON: u8 = 0x01;
const LCD_BLINKOFF: u8 = 0x00;

// Move flags
const _LCD_DISPLAYMOVE: u8 = 0x08;
const _LCD_CURSORMOVE: u8 = 0x00;
const _LCD_MOVERIGHT: u8 = 0x04;
const _LCD_MOVELEFT: u8 = 0x00;

// Function set flags
const _LCD_8BITMODE: u8 = 0x10;
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_1LINE: u8 = 0x00;
const _LCD_5X10DOTS: u8 = 0x04;
const LCD_5X8DOTS: u8 = 0x00;

// Char LCD plate GPIO numbers.
const LCD_PLATE_RS: u8 = 15;
const LCD_PLATE_RW: u8 = 14;
const LCD_PLATE_EN: u8 = 13;
const LCD_PLATE_D4: u8 = 12;
const LCD_PLATE_D5: u8 = 11;
const LCD_PLATE_D6: u8 = 10;
const LCD_PLATE_D7: u8 = 9;
const LCD_PLATE_RED: u8 = 6;
const LCD_PLATE_GREEN: u8 = 7;
const LCD_PLATE_BLUE: u8 = 8;
const _LCD_BACKPACK_LITE: u8 = 7;

// Char LCD plate button names.
const SELECT: u8 = 0;
const RIGHT: u8 = 1;
const DOWN: u8 = 2;
const UP: u8 = 3;
const LEFT: u8 = 4;

pub struct AdafruitDisplay {
    rs: u8,
    en: u8,
    d4: u8,
    d5: u8,
    d6: u8,
    d7: u8,
    displaycontrol: u8,
    displayfunction: u8,
    displaymode: u8,
    backlight: bool,
    blpol: bool,
    gpio: MCP230xx,
    cols: u8,
    lines: u8,
}

/// Based on the [Python driver by Adafruit](https://github.com/adafruit/Adafruit_Python_CharLCD)
impl AdafruitDisplay {
    /// Initialize the LCD.  RS, EN, and D4...D7 parameters should be the pins
    /// connected to the LCD RS, clock enable, and data line 4 through 7 connections.
    ///
    /// The LCD will be used in its 4-bit mode so these 6 lines are the only ones
    /// required to use the LCD.  You must also pass in the number of columns and
    /// lines on the LCD.  
    ///
    /// If you would like to control the backlight, pass in the pin connected to
    /// the backlight with the backlight parameter. The invert_polarity boolean
    /// controls if the backlight is one with a LOW signal or HIGH signal.  The
    /// default invert_polarity value is true, i.e. the backlight is on with a
    /// LOW signal.
    ///
    /// You can enable PWM of the backlight pin to have finer control on the
    /// brightness.  To enable PWM make sure your hardware supports PWM on the
    /// provided backlight pin and set enable_pwm to True (the default is False).
    /// The appropriate PWM library will be used depending on the platform, but
    /// you can provide an explicit one with the pwm parameter.
    ///
    /// The initial state of the backlight is ON, but you can set it to an
    /// explicit initial state with the initial_backlight parameter (0 is off,
    /// 1 is on/full bright).
    ///
    /// You can optionally pass in an explicit GPIO class,
    /// for example if you want to use an MCP230xx GPIO extender.  If you don't
    /// pass in an GPIO instance, the default GPIO for the running platform will
    /// be used.
    fn new(
        cols: u8,
        lines: u8,
        backlight: bool,
        invert_backlight_polarity: bool,
        gpio: MCP230xx,
    ) -> Result<Self, CommunicationError> {
        let mut display = Self {
            rs: LCD_PLATE_RS,
            en: LCD_PLATE_EN,
            d4: LCD_PLATE_D4,
            d5: LCD_PLATE_D5,
            d6: LCD_PLATE_D6,
            d7: LCD_PLATE_D7,
            displaycontrol: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            displayfunction: LCD_4BITMODE | LCD_1LINE | LCD_2LINE | LCD_5X8DOTS,
            displaymode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
            backlight: backlight,
            gpio: gpio,
            blpol: !invert_backlight_polarity,
            cols,
            lines,
        };
        display.gpio.setup(LCD_PLATE_RW, Mode::Output)?;
        display.gpio.output(LCD_PLATE_RW, false)?;
        for button in &[SELECT, RIGHT, DOWN, UP, LEFT] {
            display.gpio.setup(*button, Mode::Input)?;
            //display.gpio.pullup(button, PullUpDown::PullUp);
        }

        // Setup all pins as OUTPUT
        for pin in &[
            display.rs, display.en, display.d4, display.d5, display.d6, display.d7,
        ] {
            display.gpio.setup(*pin, Mode::Output)?;
        }

        display.write8(0x33, false)?;
        display.write8(0x32, false)?;

        let displaycontrol = LCD_DISPLAYCONTROL | display.displaycontrol;
        let displayfunction = LCD_FUNCTIONSET | display.displayfunction;
        let displaymode = LCD_ENTRYMODESET | display.displaymode;
        display.write8(displaycontrol, false)?;
        display.write8(displayfunction, false)?;
        display.write8(displaymode, false)?;
        display.clear()?;

        // Setup backlight pins
        if display.backlight {
            // display.gpio.setup(LCD_BACKPACK_LITE, Mode::Output)?;
            // display.set_backlight(1);
            display.set_color(255, 255, 255)?;
        }

        Ok(display)
    }

    /// Initializes the driver for the "Adafruit i2c 16x2 RGB LCD Pi Plate"
    pub fn for_backplate() -> Result<Self, CommunicationError> {
        AdafruitDisplay::new(16, 2, true, true, MCP230xx::for_mcp23017()?)
    }

    /// Write 8-bit value in character or data mode. Value should be an int
    /// value from 0-255, and char_mode is true if character data or false if
    /// non-character data (default).
    fn write8(&mut self, value: u8, char_mode: bool) -> Result<(), CommunicationError> {
        // waiting one millisecond to prevent writing too quickly.
        helpers::delay_microseconds(1);

        // Set character / data bit.
        self.gpio.output(self.rs, char_mode)?;

        // Write upper 4 bits
        self.gpio.output_pins(&[
            (self.d7, ((value >> 7) & 1) > 0),
            (self.d6, ((value >> 6) & 1) > 0),
            (self.d5, ((value >> 5) & 1) > 0),
            (self.d4, ((value >> 4) & 1) > 0),
        ])?;

        self.pulse_enable()?;

        // Write lower 4 bits
        self.gpio.output_pins(&[
            (self.d7, ((value >> 3) & 1) > 0),
            (self.d6, ((value >> 2) & 1) > 0),
            (self.d5, ((value >> 1) & 1) > 0),
            (self.d4, (value & 1) > 0),
        ])?;

        self.pulse_enable()?;
        Ok(())
    }

    /// Enable or disable the backlight. If PWM is not enabled (default)
    /// non-zero backlight value will turn on the backlight and a zero value
    /// will turn it off.
    pub fn set_color(&mut self, r: u8, g: u8, b: u8) -> Result<(), CommunicationError> {
        // TODO: implement PWM
        self.gpio.setup(LCD_PLATE_RED, Mode::Output)?;
        self.gpio.setup(LCD_PLATE_GREEN, Mode::Output)?;
        self.gpio.setup(LCD_PLATE_BLUE, Mode::Output)?;
        self.gpio.output_pins(&[
            (
                LCD_PLATE_RED,
                if r == 255 { self.blpol } else { !self.blpol },
            ),
            (
                LCD_PLATE_GREEN,
                if g == 255 { self.blpol } else { !self.blpol },
            ),
            (
                LCD_PLATE_BLUE,
                if b == 255 { self.blpol } else { !self.blpol },
            ),
        ])
    }

    /// Move the cursor back to its start point (upper-left corner).
    pub fn home(&mut self) -> Result<(), CommunicationError> {
        self.write8(LCD_RETURNHOME, false)?;
        helpers::delay_microseconds(3000);
        Ok(())
    }

    /// Enables auto-scroll when lines are too long.
    pub fn autoscroll(&mut self, autoscroll: bool) -> Result<(), CommunicationError> {
        if autoscroll {
            self.displaymode |= LCD_ENTRYSHIFTINCREMENT;
        } else {
            self.displaymode &= !LCD_ENTRYSHIFTINCREMENT;
        }
        let displaymode = LCD_ENTRYMODESET | self.displaymode;
        self.write8(displaymode, false)
    }

    /// Enables cursor blinking, a-la MS-DOS
    pub fn blink(&mut self, blink: bool) -> Result<(), CommunicationError> {
        if blink {
            self.displaycontrol |= LCD_BLINKON;
        } else {
            self.displaycontrol &= !LCD_BLINKON;
        }
        let displaycontrol = LCD_DISPLAYCONTROL | self.displaycontrol;
        self.write8(displaycontrol, false)
    }

    /// Fill one of the first 8 CGRAM locations with custom characters.
    /// The location parameter should be between 0 and 7 and pattern should
    /// provide an array of 8 bytes containing the pattern. E.g. you can easyly
    /// design your custom character at http://www.quinapalus.com/hd44780udg.html
    /// To show your custom character use eg. lcd.message('\x01')
    pub fn create_char(
        &mut self,
        mut location: u8,
        pattern: [u8; 8],
    ) -> Result<char, CommunicationError> {
        location &= 0x7;
        self.write8(LCD_SETCGRAMADDR | (location << 3), false)?;
        for i in 0..8 {
            self.write8(pattern[i], true)?;
        }
        Ok(char::from_u32(location as u32).unwrap())
    }

    /// Write text to display. Note that text can include newlines.
    pub fn message(&mut self, text: &str) -> Result<(), CommunicationError> {
        let mut line = 0;
        for c in text.chars() {
            let col = if self.displaymode & LCD_ENTRYLEFT > 0 {
                0
            } else {
                self.cols - 1
            };
            if c == '\n' {
                line += 1;
                self.set_cursor(col, line)?;
            } else {
                self.write8(c as u8, true)?;
            }
        }
        Ok(())
    }

    /// Move the cursor to an explicit column and row position.
    pub fn set_cursor(&mut self, col: u8, mut line: u8) -> Result<(), CommunicationError> {
        if line > self.lines {
            line = self.lines - 1;
        }
        let row = [0x00, 0x40, 0x14, 0x54][line as usize];
        self.write8(LCD_SETDDRAMADDR | (col + row), false)
    }

    /// Pulse the clock enable line off, on, off to send command.
    fn pulse_enable(&mut self) -> Result<(), CommunicationError> {
        self.gpio.output(self.en, false)?;
        helpers::delay_microseconds(1); // enable pulse must be > 450ns
        self.gpio.output(self.en, true)?;
        helpers::delay_microseconds(1); // enable pulse must be > 450ns
        self.gpio.output(self.en, false)?;
        helpers::delay_microseconds(1); // commands need > 37us to settle
        Ok(())
    }

    /// Clear the LCD
    pub fn clear(&mut self) -> Result<(), CommunicationError> {
        self.write8(LCD_CLEARDISPLAY, false)?;
        helpers::delay_microseconds(3000); // 3000 microsecond sleep, clearing the display takes a long time
        Ok(())
    }
}
