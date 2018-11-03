use rustberrypi::errors::CommunicationError;
use rustberrypi::i2c::led::{ BicolorMatrix8x8, Color };
use rustberrypi::i2c::temperature::AM2320;

struct Font;
impl Font {
    pub fn from_u8(value: u8, device: &mut BicolorMatrix8x8) -> Result<(), CommunicationError> {
        for (offset, c) in value.to_string().chars().enumerate() {
            for (i, px) in Font::get_char(c).iter().enumerate() {
                let x = (i % 3) + (offset * 3) + offset;
                let y = i / 3;
                device.set_pixel(
                    x as u8,
                    y as u8,
                    if *px == 1 { Color::Green } else { Color::Off },
                )?;
            }
        }
        device.write_display()
    }

    fn get_char(value: char) -> [u8; 15] {
        match value {
            '0' => [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1],
            '1' => [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0],
            '2' => [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1],
            '3' => [1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1],
            '4' => [1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1],
            '5' => [1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1],
            '6' => [1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1],
            '7' => [1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
            '8' => [1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1],
            '9' => [1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1],
            _ => panic!("Should be > 0 and <= 9"),
        }
    }
}

fn main() -> Result<(), CommunicationError> {
    let mut device = BicolorMatrix8x8::new()?;
    loop {
        let sensor_readings = AM2320::read()?;
        Font::from_u8(sensor_readings.temperature as u8, &mut device)?;
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}