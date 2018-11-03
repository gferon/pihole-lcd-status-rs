use bmp;
use rppal::i2c;

#[derive(Debug)]
pub enum CommunicationError {
    BusError(i2c::Error),
    WrongPin(u8),
    BitmapError(bmp::BmpError),
    ReadingError,
    WritingError,
}
