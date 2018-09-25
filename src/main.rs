extern crate rppal;

use rppal::i2c::I2c;


// Constants
const MCP23017_IOCON_BANK0: u8 = 0x0A; // IOCON when Bank 0 active
const MCP23017_IOCON_BANK1: u8 = 0x15; // IOCON when Bank 1 active

// These are register addresses when in Bank 1 only:
const MCP23017_GPIOA          :u8 = 0x09;
const MCP23017_IODIRB         :u8 = 0x10;
const MCP23017_GPIOB          :u8 = 0x19;

// Port expander input pin definitions
const SELECT                  :u8 = 0;
const RIGHT                   :u8 = 1;
const DOWN                    :u8 = 2;
const UP                      :u8 = 3;
const LEFT                    :u8 = 4;

// LED colors
const OFF                     :u8 = 0x00;
const RED                     :u8 = 0x01;
const GREEN                   :u8 = 0x02;
const BLUE                    :u8 = 0x04;
const YELLOW                  :u8 = RED + GREEN;
const TEAL                    :u8 = GREEN + BLUE;
const VIOLET                  :u8 = RED + BLUE;
const WHITE                   :u8 = RED + GREEN + BLUE;
const ON                      :u8 = RED + GREEN + BLUE;

// LCD Commands
const LCD_CLEARDISPLAY        :u8 = 0x01;
const LCD_RETURNHOME          :u8 = 0x02;
const LCD_ENTRYMODESET        :u8 = 0x04;
const LCD_DISPLAYCONTROL      :u8 = 0x08;
const LCD_CURSORSHIFT         :u8 = 0x10;
const LCD_FUNCTIONSET         :u8 = 0x20;
const LCD_SETCGRAMADDR        :u8 = 0x40;
const LCD_SETDDRAMADDR        :u8 = 0x80;

// Flags for display on/off control
const LCD_DISPLAYON           :u8 = 0x04;
const LCD_DISPLAYOFF          :u8 = 0x00;
const LCD_CURSORON            :u8 = 0x02;
const LCD_CURSOROFF           :u8 = 0x00;
const LCD_BLINKON             :u8 = 0x01;
const LCD_BLINKOFF            :u8 = 0x00;

// Flags for display entry mode
const LCD_ENTRYRIGHT          :u8 = 0x00;
const LCD_ENTRYLEFT           :u8 = 0x02;
const LCD_ENTRYSHIFTINCREMENT :u8 = 0x01;
const LCD_ENTRYSHIFTDECREMENT :u8 = 0x00;

// Flags for display/cursor shift
const LCD_DISPLAYMOVE :u8 = 0x08;
const LCD_CURSORMOVE  :u8 = 0x00;
const LCD_MOVERIGHT   :u8 = 0x04;
const LCD_MOVELEFT    :u8 = 0x00;

// Line addresses for up to 4 line displays.  Maps line number to DDRAM address for line.
// const LINE_ADDRESSES :u8 = { 1: 0xC0, 2: 0x94, 3: 0xD4 };

// Truncation constants for message function truncate parameter.
const NO_TRUNCATE       :u8 = 0;
const TRUNCATE          :u8 = 1;
const TRUNCATE_ELLIPSIS :u8 = 2;

const FLIP: [u8; 16] = [0b00000000, 0b00010000, 0b00001000, 0b00011000,
             0b00000100, 0b00010100, 0b00001100, 0b00011100,
             0b00000010, 0b00010010, 0b00001010, 0b00011010,
0b00000110, 0b00010110, 0b00001110, 0b00011110];

struct AdafruitDisplay {
    i2c: I2c,
    porta: u8,
    portb: u8,
    ddrb: u8,
    displayshift: u8,
    displaymode: u8,
    displaycontrol: u8
}

impl AdafruitDisplay {

    fn new() -> Self {
        // I2C is relatively slow.  MCP output port states are cached
        // so we don't need to constantly poll-and-change bit states.
        Self {
            i2c: I2c::with_bus(1).unwrap(),
            porta: 0,
            portb: 0,
            ddrb: 0b00000010,
            displayshift: LCD_CURSORMOVE | LCD_MOVERIGHT,
            displaymode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
            displaycontrol: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
        }
    }

    fn init(&mut self) {
        let backlight = ON;
        //Set initial backlight color.
        let c = !backlight;
        self.porta = (self.porta & 0b00111111) | ((c & 0b011) << 6);
        self.portb = (self.portb & 0b11111110) | ((c & 0b100) >> 2);

        // Set MCP23017 IOCON register to Bank 0 with sequential operation.
        // If chip is already set for Bank 0, this will just write to OLATB,
        // which won't seriously bother anything on the plate right now
        // (blue backlight LED will come on, but that's done in the next
        // step anyway).
        self.i2c.smbus_write_byte(MCP23017_IOCON_BANK1, 0);
        
        // Brute force reload ALL registers to known state.  This also
        // sets up all the input pins, pull-ups, etc. for the Pi Plate.
        let registers = [
            0b00111111,   // IODIRA    R+G LEDs=outputs, buttons=inputs
            self.ddrb ,   // IODIRB    LCD D7=input, Blue LED=output
            0b00111111,   // IPOLA     Invert polarity on button inputs
            0b00000000,   // IPOLB
            0b00000000,   // GPINTENA  Disable interrupt-on-change
            0b00000000,   // GPINTENB
            0b00000000,   // DEFVALA
            0b00000000,   // DEFVALB
            0b00000000,   // INTCONA
            0b00000000,   // INTCONB
            0b00000000,   // IOCON
            0b00000000,   // IOCON
            0b00111111,   // GPPUA     Enable pull-ups on buttons
            0b00000000,   // GPPUB
            0b00000000,   // INTFA
            0b00000000,   // INTFB
            0b00000000,   // INTCAPA
            0b00000000,   // INTCAPB
            self.porta,   // GPIOA
            self.portb,   // GPIOB
            self.porta,   // OLATA
            self.portb ];
        for register in registers.iter() {
            self.i2c.smbus_write_byte(0, *register);
        }

        // Switch to Bank 1 and disable sequential operation.
        // From this point forward, the register addresses do NOT match
        // the list immediately above.  Instead, use the constants defined
        // at the start of the class.  Also, the address register will no
        // longer increment automatically after this -- multi-byte
        // operations must be broken down into single-byte calls.
        self.i2c.smbus_write_byte(MCP23017_IOCON_BANK0, 0b10100000);

        self.write(0x33); // Init
        self.write(0x32); // Init
        self.write(0x28); // 2 line 5x8 matrix
        self.write(LCD_CLEARDISPLAY);
        let shift = LCD_CURSORSHIFT | self.displayshift;
        self.write(shift);
        let mode = LCD_ENTRYMODESET | self.displaymode;
        self.write(mode);
        let control = LCD_DISPLAYCONTROL | self.displaycontrol;
        self.write(control);
        self.write(LCD_RETURNHOME);
    }

    fn prepare_write(&mut self, char_mode: bool) -> u8 {
        // If pin D7 is in input state, poll LCD busy flag until clear.
        if self.ddrb & 0b00000010 == 1 {
            let lo = (self.portb & 0b00000001) | 0b01000000;
            let hi = lo | 0b00100000; // E=1 (strobe)
            self.i2c.smbus_write_byte(MCP23017_GPIOB, lo);
            // loop {
            //     let mut bits = [0u8, 1];
            //     // Strobe high (enable)
            //     // First nybble contains busy state
            //     self.i2c.write_read(&[hi], &bits);
            //     //bits = self.i2c.bus.read_byte(self.i2c.address)
            //     // Strobe low, high, low.  Second nybble (A3) is ignored.
            //     self.i2c.block_write(MCP23017_GPIOB, [lo, hi, lo]);
            //     if (bits & 0b00000010) == 0 { break; } // D7=0, not busy
            // }
            self.portb = lo;

            // Polling complete, change D7 pin to output
            self.ddrb &= 0b11111101;
            self.i2c.smbus_write_byte(MCP23017_IODIRB, self.ddrb);
        }

        let mut bitmask = self.portb & 0b00000001;   // Mask out PORTB LCD control bits
        if char_mode {
            bitmask |= 0b10000000 // Set data bit if not a command
        }
        bitmask
    }
    
    /// Write a single byte to the LCD
    fn write(&mut self, value: u8) {
        let char_mode = false;
        let bitmask = self.prepare_write(char_mode);
        let data = self.out4(bitmask, value);
        self.i2c.smbus_block_write(MCP23017_GPIOB, &data);
        self.portb = *data.last().unwrap();
    }

    fn out4(&mut self, bitmask: u8, value: u8) -> [u8; 4] {
        let hi = bitmask | FLIP[(value >> 4) as usize];
        let lo = bitmask | FLIP[(value & 0x0F) as usize];
        [hi | 0b00100000, hi, lo | 0b00100000, lo]
    }
}

fn main() {
    let mut display = AdafruitDisplay::new();
    display.init();

    println!("Hello, world!");
}
