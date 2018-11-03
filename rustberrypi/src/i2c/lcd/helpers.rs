use bmp;
use crate::errors::CommunicationError;
use crate::i2c::lcd::AdafruitDisplay;

use std::path::PathBuf;
use std::{thread, time};

pub fn load_characters_from_bmp(
    display: &mut AdafruitDisplay,
    filepath: PathBuf,
) -> Result<(), CommunicationError> {
    let img = bmp::open(filepath).map_err(|e| CommunicationError::BitmapError(e))?;

    let mut data = [
        [0; 8], [0; 8], [0; 8], [0; 8], [0; 8], [0; 8], [0; 8], [0; 8],
    ];

    for y in 0..16 {
        for x in 0..16 {
            let char_x = x % 5;
            let char_idx = (x / 5) + 4 * (y / 8);

            let pixel = img.get_pixel(x as u32, y as u32);
            let is_black = pixel.r < 240 && pixel.g < 240 && pixel.b < 240;
            if is_black {
                data[char_idx as usize][(y % 8) as usize] |= 1 << (5 - char_x);
            }
        }
    }

    load_16px_block(display, data)
}

pub fn load_ferris(display: &mut AdafruitDisplay) -> Result<(), CommunicationError> {
    load_16px_block(
        display,
        [
            [0x0, 0x3, 0xf, 0xf, 0xf, 0x1e, 0x18, 0x18],
            [0x0, 0x18, 0x0, 0x18, 0x10, 0x0, 0x0, 0x0],
            [0x0, 0xf, 0x3, 0xf, 0x7, 0x1, 0x0, 0x0],
            [0x0, 0x0, 0x18, 0x18, 0x18, 0x1c, 0xc, 0xc],
            [0x19, 0x1f, 0xf, 0x3, 0xf, 0x19, 0x3, 0x6],
            [0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0xf],
            [0x1e, 0x1f, 0x1f, 0x1f, 0x1f, 0x1e, 0x1f, 0x19],
            [0xc, 0x1c, 0x18, 0x0, 0x18, 0xc, 0x0, 0x10],
        ],
    )
}

pub fn load_16px_block(
    display: &mut AdafruitDisplay,
    data: [[u8; 8]; 8],
) -> Result<(), CommunicationError> {
    for (i, block) in data.iter().enumerate() {
        display.create_char(i as u8, *block)?;
    }
    Ok(())
}

pub fn delay_microseconds(duration: u64) {
    thread::sleep(time::Duration::from_micros(duration));
}
