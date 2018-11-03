use rustberrypi::errors::CommunicationError;
use rustberrypi::i2c::lcd::AdafruitDisplay;
use rustberrypi::i2c::lcd;
use rustberrypi::i2c::lcd::Button;

use serde_derive::Deserialize;
use ureq;

use std::char;
use std::sync::{Arc, Mutex};
use std::thread;

fn display_ferris(display: &mut AdafruitDisplay) -> Result<(), CommunicationError> {
    lcd::helpers::load_ferris(display)?;

    display.clear()?;
    display.home()?;

    display.message(&format!(
        "{}{}{}{} Pi-hole",
        char::from_u32(0).unwrap(),
        char::from_u32(1).unwrap(),
        char::from_u32(2).unwrap(),
        char::from_u32(3).unwrap()
    ))?;

    display.set_cursor(0, 1)?;
    display.message(&format!(
        "{}{}{}{} Status",
        char::from_u32(4).unwrap(),
        char::from_u32(5).unwrap(),
        char::from_u32(6).unwrap(),
        char::from_u32(7).unwrap()
    ))?;

    Ok(())
}

fn get_pihole_status() -> Result<PiHoleStatus, PiHoleError> {
    let status: PiHoleStatus = serde_json::from_str(
        &ureq::get("http://192.168.188.20/admin/api.php")
            .call()
            .into_string()?,
    )?;
    Ok(status)
}

fn main() -> Result<(), PiHoleError> {
    let display = Arc::new(Mutex::new(AdafruitDisplay::for_backplate()?));
    let d = display.clone();
    ctrlc::set_handler(move || {
        &mut d
            .lock()
            .map_err(|_| panic!("Could not lock access to display."))
            .unwrap()
            .set_color(0, 0, 0);
        std::process::exit(1);
    }).expect("Error setting Ctrl-C handler");

    let d1 = display.clone();
    thread::spawn(move || loop {
        let status: PiHoleStatus = get_pihole_status().unwrap();
        display_ferris(
            &mut d1
                .clone()
                .lock()
                .map_err(|_| panic!("Could not lock access to display."))
                .unwrap(),
        ).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));

        {
            let display = &mut d1
                .lock()
                .map_err(|_| panic!("Could not lock access to display."))
                .unwrap();
            display.clear().unwrap();
            display
                .message(&format!(
                    "DNS last 24h\n{} queries",
                    status.dns_queries_today
                )).unwrap();
        }

        std::thread::sleep(std::time::Duration::from_secs(10));

        {
            let display = &mut d1
                .lock()
                .map_err(|_| panic!("Could not lock access to display."))
                .unwrap();
            display.clear().unwrap();
            display
                .message(&format!(
                    "Blocked {} ads\n{:.1}% less junk",
                    status.ads_blocked_today, status.ads_percentage_today,
                )).unwrap();
        }

        std::thread::sleep(std::time::Duration::from_secs(10));
    });

    thread::spawn(move || loop {
        {
            let display = &mut display
                .lock()
                .map_err(|_| panic!("Could not lock access to display."))
                .unwrap();
            if display.is_pressed(Button::Select).unwrap() {
                display.toggle_backlight().unwrap();
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(200));
    }).join()
    .unwrap();

    Ok(())
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
    DeviceError(CommunicationError),
}

impl From<serde_json::Error> for PiHoleError {
    fn from(err: serde_json::Error) -> PiHoleError {
        PiHoleError::DataError(err)
    }
}

impl From<rustberrypi::CommunicationError> for PiHoleError {
    fn from(err: CommunicationError) -> PiHoleError {
        PiHoleError::DeviceError(err)
    }
}

impl From<std::io::Error> for PiHoleError {
    fn from(err: std::io::Error) -> PiHoleError {
        PiHoleError::HttpError(err)
    }
}
