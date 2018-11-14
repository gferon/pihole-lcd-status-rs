use failure::Error;
use influx_db_client::{Client, Point, Precision, Value};
use rustberrypi::i2c::temperature::AM2320;

pub fn send_sensor_data(client: Client, measurement: &str, tag: String) -> Result<(), Error> {
    let sensor_readings = AM2320::read()?;
    let point = Point::new(measurement)
        .add_tag("tags", Value::String(tag))
        .add_field("temperature", Value::Float(sensor_readings.temperature))
        .add_field("humidity", Value::Float(sensor_readings.humidity))
        .to_owned();
    let _ = client.write_point(point, Some(Precision::Seconds), None)?;
    println!("Sent {:?} to Grafana!", sensor_readings);

    Ok(())
}
