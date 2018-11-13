use rustberrypi::i2c::temperature::AM2320;

use failure::Error;
use influx_db_client::{Client, Point, Precision, Value};
use structopt::StructOpt;
use url::Url;

mod errors;
mod weather;

#[derive(StructOpt, Debug)]
#[structopt(name = "winterr")]
struct Opt {
    #[structopt(short = "H", long = "host", parse(try_from_str = "Url::parse"))]
    // Address of the InfluxDB instance to connect to
    host: Url,

    #[structopt(long = "db", default_value = "home")]
    db: String,

    #[structopt(short = "m", long = "measurement", default_value = "am2320")]
    // Named of the InfluxDB measurement
    measurement: String,

    // Tag to use when sending data over to Grafana
    #[structopt(short = "t", long = "tag")]
    tag: String,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "sensor")]
    Sensor,
    #[structopt(name = "weather")]
    Weather {
        #[structopt(short = "k", long = "key")]
        // Your OpenWeatherMap API key
        api_key: String,
    },
}

fn get_client(host: &Url, db: &str) -> Client {
    let client = Client::new(host.as_str(), db);
    if let Some(password) = host.password() {
        client.set_authentication(host.username(), password)
    } else {
        client
    }
}

fn send_sensor_data(client: Client, measurement: &str, tag: String) -> Result<(), Error> {
    // prepare measurement
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

fn main() -> Result<(), Error> {
    env_logger::init();
    let opt = Opt::from_args();

    match opt.command {
        Command::Sensor => {
            send_sensor_data(get_client(&opt.host, &opt.db), &opt.measurement, opt.tag)
        }
        Command::Weather { api_key } => weather::send_current_weather(
            get_client(&opt.host, &opt.db),
            opt.measurement,
            opt.tag,
            api_key,
        ),
    }
}
