use rustberrypi::errors::CommunicationError;
use rustberrypi::i2c::led::{BicolorMatrix8x8, Color};
use rustberrypi::i2c::temperature::AM2320;

use influx_db_client::{Client, Point, Precision, Value};

use env_logger;
use failure::Error;
use log::error;

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

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut device = BicolorMatrix8x8::new()?;
    loop {
        let sensor_readings = AM2320::read()?;

        let client = Client::new("http://192.168.188.20:8086", "home")
            .set_authentication("winterr", "iscoming");

        // prepare measurement
        let point = Point::new("am2320")
            .add_tag("tags", Value::String("bedroom".into()))
            .add_field("temperature", Value::Float(sensor_readings.temperature))
            .add_field("humidity", Value::Float(sensor_readings.humidity))
            .to_owned();
        let _ = client
            .write_point(point, Some(Precision::Seconds), None)
            .map_err(|e| error!("Could not send data to Grafana, will retry later. {}", e));

        // blink one pixel to show the program is alive
        device.set_pixel(7, 7, Color::Yellow, true)?;
        std::thread::sleep(std::time::Duration::from_millis(200));
        device.set_pixel(7, 7, Color::Off, true)?;

        // display on LED display
        Font::from_u8(sensor_readings.temperature.floor() as u8, &mut device)?;
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
