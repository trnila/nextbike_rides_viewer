use env_logger::Logger;
use indicatif::ProgressBar;
use log::Level;
use log::LevelFilter;
use log::Metadata;
use logging::LoggingAwareProgressBar;
use rides::Rides;
use serde::{Deserialize, Serialize};
use stations::Stations;
use std::collections::hash_map::Entry;
use std::fmt::Display;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::PathBuf;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::thread;
use std::time;
use std::time::Duration;
use std::time::SystemTime;
use regex::Regex;
use log::{debug, error};

use clap::{Parser, Subcommand};


mod input;
mod offline;
mod processor;
mod stations;
mod rides;
mod logging;

use offline::*;
use input::*;
use processor::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    station: String,
    station_uid: StationId,
    timestamp: u64,
}

#[derive(Debug)]
enum ParseDurationError {
    ParseIntError(std::num::ParseIntError),
    InvalidUnitError(char),
    NoDurationError,
}

impl std::error::Error for ParseDurationError {}

impl Display for ParseDurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseDurationError::ParseIntError(_err) => write!(f, "Invalid number"),
            ParseDurationError::InvalidUnitError(unit) => write!(f, "Invalid unit: {unit}"),
            ParseDurationError::NoDurationError => write!(f, "No duration provided"),
        }
    }
}

impl From<std::num::ParseIntError> for ParseDurationError {
    fn from(err: std::num::ParseIntError) -> Self {
        ParseDurationError::ParseIntError(err)
    }
}

fn parse_duration(arg: &str) -> Result<std::time::Duration, ParseDurationError> {
    match arg.chars().last() {
        None => Err(ParseDurationError::NoDurationError),
        Some(c) => {
            if c.is_digit(10) {
                let value: u64 = arg.parse()?;
                return Ok(Duration::from_secs(value));
            }

            let value = arg[0 .. arg.len() - 1].parse()?;
            match c {
                's' => Ok(Duration::from_secs(value)),
                'm' => Ok(Duration::from_secs(value * 60)),
                'h' => Ok(Duration::from_secs(value * 60 * 60)),
                'd' => Ok(Duration::from_secs(value * 60 * 60 * 24)),
                _ => Err(ParseDurationError::InvalidUnitError(c)),
            }
        }
    }
}

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    #[clap(short, long, help="path to output JSON", default_value="./viewer/public/rides.json")]
    rides_path: PathBuf,

    #[clap(short, long, help="path to output JSON", default_value="./viewer/public/stations.json")]
    stations_path: PathBuf,

    #[clap(short, long, help="directory with input JSONs", default_value="./data/")]
    input_dir: PathBuf,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Online {
        #[clap(parse(try_from_str = parse_duration))]
        interval: std::time::Duration,
    },

    Offline,
}

fn main() {
    logging::Logger::init();
    let cli = Cli::parse();

    let mut stations = Stations::new(cli.stations_path);

    match cli.command {
        Commands::Online { interval } => {
            let mut state = HashMap::<u32, Record>::new();
            let mut rides = Rides::new_blank(&cli.rides_path);

            loop {
                scrap_data(&cli.input_dir, &mut state, &mut stations, &mut rides);               
                thread::sleep(interval);
            }

        },
        Commands::Offline => {
            load_from_disk(&cli.input_dir, &cli.rides_path, &mut stations);
        }
    }
}

fn scrap_data(input_dir: &PathBuf, state: &mut HashMap<u32, Record>, stations: &mut Stations, rides: &mut Rides) {
    let ts = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs();

    debug!("Downloading new data at {ts}");
    match reqwest::blocking::get("https://api.nextbike.net/maps/nextbike-live.json?city=271") {
        Ok(resp) => {
            match resp.text() {
                Ok(body) => {
                    let path = input_dir.join(format!("{ts}.json"));

                    if let Err(err) = fs::write(&path, &body) {
                        error!("Failed to store response {path:?}: {err}");
                    }

                    match serde_json::from_str::<JSON>(&body) {
                        Ok(json) => {
                            let rides = process(ts, &json, state, stations, rides);
                            debug!("{rides} new rides found");
                        }
                        Err(err) => error!("Failed to parse json: {err}"),
                    }                                
                },
                Err(err) => error!("Failed to parse response: {err}"),
            }
        },
        Err(err) => error!("Failed to fetch: {err}"),
    }
}
