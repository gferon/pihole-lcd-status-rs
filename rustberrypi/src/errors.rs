use bmp;
use rppal::i2c;

use std::fmt;
use std::error;

#[derive(Debug)]
pub enum CommunicationError {
    BusError(i2c::Error),
    WrongPin(u8),
    BitmapError(bmp::BmpError),
    ReadingError,
    WritingError,
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for CommunicationError {
    fn description(&self) -> &str {
        "there was a communication error with a device connected to the Raspberry Pi."
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
