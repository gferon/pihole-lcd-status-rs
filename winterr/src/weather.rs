use failure::Error;
use hyper;
use hyper::rt::{self, Future, Stream};
use hyper_tls::HttpsConnector;
use influx_db_client::{Client, Point, Precision, Value};
use log::error;
use serde_derive::Deserialize;

fn get_current_weather(url: hyper::Uri) -> impl Future<Item = Weather, Error = FetchError> {
    let https = HttpsConnector::new(4).unwrap();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    client
        .get(url)
        .and_then(|res| res.into_body().concat2())
        .from_err::<FetchError>()
        .and_then(|body| {
            let resp: Response = serde_json::from_slice(&body)?;
            Ok(resp.main)
        })
        .from_err()
}

pub fn send_current_weather(
    client: Client,
    measurement: String,
    api_key: String,
) -> Result<(), Error> {
    // get weather from OpenWeather API
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?APPID={}&q=Berlin,DE",
        api_key
    )
    .parse()
    .unwrap();

    let fut = get_current_weather(url)
        .map(move |weather| {
            println!("Got weather info: {:?}", weather);
            let point = Point::new(&measurement)
                .add_tag("tags", Value::String("weather"))
                .add_field("temperature", Value::Float(weather.temperature()))
                .add_field("humidity", Value::Float(weather.humidity))
                .to_owned();

            let _ = client
                .write_point(point, Some(Precision::Seconds), None)
                .map_err(|e| error!("Could not send data to Grafana, will retry later. {}", e));
        })
        .map_err(|e| match e {
            FetchError::Http(e) => eprintln!("http error: {}", e),
            FetchError::Json(e) => eprintln!("json parsing error: {}", e),
        });

    rt::run(fut);
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Response {
    main: Weather,
}

#[derive(Deserialize, Debug)]
struct Weather {
    pub humidity: f64,
    #[serde(rename = "temp")]
    temperature: f64,
}

impl Weather {
    fn temperature(&self) -> f64 {
        self.temperature - 273.15
    }
}

// Define a type so we can return multiple types of errors
enum FetchError {
    Http(hyper::Error),
    Json(serde_json::Error),
}

impl From<hyper::Error> for FetchError {
    fn from(err: hyper::Error) -> FetchError {
        FetchError::Http(err)
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(err: serde_json::Error) -> FetchError {
        FetchError::Json(err)
    }
}
