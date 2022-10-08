use log::{debug, error};

use std::fmt::Display;
use std::path::PathBuf;
use std::thread;
use std::time;
use std::time::Duration;
use std::time::SystemTime;

use clap::{Parser, Subcommand};

use crate::input::JsonResponse;
use crate::processor::RidesProcessor;
use crate::rides::Rides;
use crate::stations::Stations;

mod input;
mod processor;
mod rides;
mod stations;

#[derive(Debug)]
enum ParseDurationError {
    ParseInt(std::num::ParseIntError),
    InvalidUnit(char),
    NoDuration,
}

impl std::error::Error for ParseDurationError {}

impl Display for ParseDurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseDurationError::ParseInt(_err) => write!(f, "Invalid number"),
            ParseDurationError::InvalidUnit(unit) => write!(f, "Invalid unit: {unit}"),
            ParseDurationError::NoDuration => write!(f, "No duration provided"),
        }
    }
}

impl From<std::num::ParseIntError> for ParseDurationError {
    fn from(err: std::num::ParseIntError) -> Self {
        ParseDurationError::ParseInt(err)
    }
}

fn parse_duration(arg: &str) -> Result<std::time::Duration, ParseDurationError> {
    match arg.chars().last() {
        None => Err(ParseDurationError::NoDuration),
        Some(c) => {
            if c.is_ascii_digit() {
                let value: u64 = arg.parse()?;
                return Ok(Duration::from_secs(value));
            }

            let value = arg[0..arg.len() - 1].parse()?;
            match c {
                's' => Ok(Duration::from_secs(value)),
                'm' => Ok(Duration::from_secs(value * 60)),
                'h' => Ok(Duration::from_secs(value * 60 * 60)),
                'd' => Ok(Duration::from_secs(value * 60 * 60 * 24)),
                _ => Err(ParseDurationError::InvalidUnit(c)),
            }
        }
    }
}

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    #[clap(
        short,
        long,
        help = "path to output JSON",
        default_value = "./viewer/public/rides.json"
    )]
    rides_path: PathBuf,

    #[clap(
        short,
        long,
        help = "path to output JSON",
        default_value = "./viewer/public/stations.json"
    )]
    stations_path: PathBuf,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Online {
        #[clap(parse(try_from_str = parse_duration))]
        interval: std::time::Duration,
    },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let stations = Stations::new(cli.stations_path);

    match cli.command {
        Commands::Online { interval } => {
            let mut processor =
                RidesProcessor::new(stations, Rides::new_appending(&cli.rides_path));

            loop {
                scrap_data(&mut processor);
                thread::sleep(interval);
            }
        }
    }
}

fn scrap_data(processor: &mut RidesProcessor) {
    let ts = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    debug!("Downloading new data at {ts}");
    match reqwest::blocking::get("https://api.nextbike.net/maps/nextbike-live.json?city=271") {
        Ok(resp) => match resp.text() {
            Ok(body) => {
                match serde_json::from_str::<JsonResponse>(&body) {
                    Ok(json) => {
                        let rides = processor.process(ts, &json);
                        debug!("{rides} new rides found");
                    }
                    Err(err) => error!("Failed to parse json: {err}"),
                }
            }
            Err(err) => error!("Failed to parse response: {err}"),
        },
        Err(err) => error!("Failed to fetch: {err}"),
    }
}
