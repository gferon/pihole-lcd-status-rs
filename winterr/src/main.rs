use failure::Error;
use influx_db_client::Client;
use structopt::StructOpt;
use url::Url;

mod errors;
mod sensor;
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
        #[structopt(short = "k", long = "api-key")]
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

fn main() -> Result<(), Error> {
    env_logger::init();
    let opt = Opt::from_args();

    match opt.command {
        Command::Sensor => {
            sensor::send_sensor_data(get_client(&opt.host, &opt.db), &opt.measurement, opt.tag)
        }
        Command::Weather { api_key } => {
            weather::send_current_weather(get_client(&opt.host, &opt.db), opt.measurement, api_key)
        }
    }
}
