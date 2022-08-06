use std::time::Duration;
use std::{collections::HashMap, io::BufWriter, fs::File};
use std::io::Write;

use log::{error, warn};
use regex::Regex;
use serde::{Serialize, Deserialize};

use crate::rides::Rides;
use crate::{stations::Stations, input::JSON, Record};


#[derive(Serialize, Deserialize, Debug)]
struct CsvStation {
    name: String,
    lat: f32,
    lng: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Position {
    lat: f32,
    lng: f32,
}


#[derive(Serialize, Deserialize, Debug)]
struct Locatin {
    timestamp: u64,
    name: String,
    lat: f32,
    lng: f32,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct CsvRide {
    bike_id: u32,

    src: P,
    dst: P,
}

#[derive(Serialize, Deserialize, Debug)]
struct P {
    timestamp: u64,
    name: String,
    lat: f32,
    lng: f32
}

pub struct RidesProcessor {
    stations: Stations,
    state: HashMap::<u32, Record>,
    rides: Rides,
    last_timestamp: u64,
}

impl RidesProcessor {
    pub fn new(stations: Stations, rides: Rides) -> Self {
        RidesProcessor {
            stations,
            rides,
            state: HashMap::new(),
            last_timestamp: 0,
        }
    }

    pub fn process(&mut self, timestamp: u64, json: &JSON) -> u64 {
        if json.countries.len() != 1 {
            error!("Number of countries in {timestamp} is not 1, but {}", json.countries.len());
            return 0;
        }

        let previous_timestamp = self.last_timestamp;
        self.last_timestamp = timestamp;
        if previous_timestamp != 0 {
            if timestamp < previous_timestamp {
                error!("current timestamp is lesser then previous {timestamp} < {previous_timestamp}");
                return 0;
            }

            let diff = Duration::from_secs(timestamp - previous_timestamp);
            if diff > Duration::from_secs(10 * 60) {
                warn!("Time gap of {diff:?} found, resetting state");
                self.state.clear();
                return 0;
            }
        }

        let mut rides = 0u64;
        for place in &json.countries[0].cities[0].places {
            if place.name.starts_with("BIKE") {
                continue
            }

            self.stations.add_station(place.uid, &place.name, place.lat, place.lng);

            for bike in &place.bike_list {
                let id = bike.number.parse::<u32>().unwrap();

                if let Some(rec) = self.state.get(&id) {
                    if rec.station_uid != place.uid {
                        let s = self.stations.stations.get(&rec.station_uid).unwrap();

                        self.rides.write(&CsvRide{
                            bike_id: id,
                            src: P{
                                timestamp: rec.timestamp,
                                name: clean_name(&rec.station),
                                lat: s.lat,
                                lng: s.lng,
                            },
                            dst: P {
                                timestamp: timestamp as u64,
                                name: clean_name(&place.name),
                                lat: place.lat,
                                lng: place.lng,
                            }
                        }).unwrap();

                        rides += 1;
                    }
                }

                self.state.insert(id, Record{
                    timestamp: timestamp as u64,
                    station: place.name.clone(),
                    station_uid: place.uid,
                });
            }
        }
        rides
    }
}

fn clean_name(name: &str) -> String {
    let re = Regex::new(r"\*?\(.+").unwrap();
    return re.replace(name, "").to_string().trim().to_string();
}