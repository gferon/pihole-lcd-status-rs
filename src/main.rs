use adafruit::errors::CommunicationError;
use std::char;
use serde_derive::Deserialize;
use ureq;

fn initialize_display() -> Result<adafruit::AdafruitDisplay, CommunicationError> {
    let mut display = adafruit::AdafruitDisplay::for_backplate()?;
    adafruit::helpers::load_ferris(&mut display)?;

    display.clear()?;
    display.home()?;

    display.message(&format!(
        "{}{}{}{} Pi",
        char::from_u32(0).unwrap(),
        char::from_u32(1).unwrap(),
        char::from_u32(2).unwrap(),
        char::from_u32(3).unwrap()
    ))?;

    display.set_cursor(0, 1)?;
    display.message(&format!(
        "{}{}{}{} HOLE",
        char::from_u32(4).unwrap(),
        char::from_u32(5).unwrap(),
        char::from_u32(6).unwrap(),
        char::from_u32(7).unwrap()
    ))?;

    Ok(display)
}

fn get_pihole_status() -> Result<PiHoleStatus, PiHoleError> {
    let status: PiHoleStatus = serde_json::from_str(
        &ureq::get("http://192.168.188.20/admin/api.php")
            .call()
            .into_string()?,
    )?;
    Ok(status)
}

fn display_status(mut display: adafruit::AdafruitDisplay) -> Result<(), PiHoleError> {
    let status: PiHoleStatus = get_pihole_status()?;

    display.clear()?;
    display.message(&format!(
        "DNS last 24h\n{} queries",
        status.dns_queries_today
    ))?;

    std::thread::sleep(std::time::Duration::from_secs(10));

    display.clear()?;
    display.message(&format!(
        "Blocked {} ads\n{:.1}% less junk",
        status.ads_blocked_today, status.ads_percentage_today,
    ))?;

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}

fn main() -> Result<(), PiHoleError> {
    loop {
        display_status(initialize_display()?)?;
    }
}

#[derive(Deserialize, Debug)]
struct PiHoleStatus {
    domains_being_blocked: usize,
    dns_queries_today: usize,
    ads_blocked_today: usize,
    ads_percentage_today: f32,
    unique_domains: usize,
    queries_forwarded: usize,
    queries_cached: usize,
    dns_queries_all_types: usize,
}

#[derive(Debug)]
pub enum PiHoleError {
    HttpError(std::io::Error),
    DataError(serde_json::Error),
    DeviceError(adafruit::errors::CommunicationError),
}

impl From<serde_json::Error> for PiHoleError {
    fn from(err: serde_json::Error) -> PiHoleError {
        PiHoleError::DataError(err)
    }
}

impl From<adafruit::CommunicationError> for PiHoleError {
    fn from(err: adafruit::CommunicationError) -> PiHoleError {
        PiHoleError::DeviceError(err)
    }
}

impl From<std::io::Error> for PiHoleError {
    fn from(err: std::io::Error) -> PiHoleError {
        PiHoleError::HttpError(err)
    }
}
