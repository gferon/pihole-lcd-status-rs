use crate::i2c::led::{BicolorMatrix8x8, Blink, Color, HT16K33};

fn main() -> Result<(), CommunicationError> {
    struct Font;
    impl Font {
        pub fn from_u8(value: u8, device: &mut HT16K33) -> Result<(), CommunicationError> {
            let color = match value {
                0...21 => Color::Green,
                21...25 => Color::Yellow,
                _ => Color::Red,
            };
            for (offset, c) in value.to_string().chars().enumerate() {
                for (i, px) in Font::get_char(c).iter().enumerate() {
                    let x = (i % 3) + (offset * 3) + offset;
                    let y = i / 3 + 1;
                    device.set_pixel(
                        x as u8,
                        y as u8,
                        if *px == 1 { color.clone() } else { Color::Off },
                        false,
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
}
