use std::collections::HashMap;
use std::time::Duration;

use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use log::{error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::input::{BikeId, StationId};
use crate::rides::{RideEvent, RideLocation, RidesWriter};
use crate::{input::JsonResponse, stations::Stations};

#[derive(Serialize, Deserialize, Debug)]
struct StateRecord {
    station_uid: StationId,
    timestamp: u64,
}

pub struct RidesProcessor {
    stations: Stations,
    state: HashMap<BikeId, StateRecord>,
    rides: RidesWriter,
    last_timestamp: u64,
}

impl RidesProcessor {
    pub fn new(stations: Stations, rides: RidesWriter) -> Self {
        RidesProcessor {
            stations,
            rides,
            state: HashMap::new(),
            last_timestamp: 0,
        }
    }

    pub fn process(&mut self, timestamp: u64, json: &JsonResponse) -> u64 {
        if json.countries.len() != 1 {
            error!(
                "Number of countries in {timestamp} is not 1, but {}",
                json.countries.len()
            );
            return 0;
        }

        let previous_timestamp = self.last_timestamp;
        self.last_timestamp = timestamp;
        if previous_timestamp != 0 {
            if timestamp < previous_timestamp {
                error!(
                    "current timestamp is lesser then previous {timestamp} < {previous_timestamp}"
                );
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
        for cur in &json.countries[0].cities[0].places {
            if cur.name.starts_with("BIKE") {
                continue;
            }

            self.stations
                .add_station(cur.uid, &cur.name, cur.lat, cur.lng);

            for bike in &cur.bike_list {
                if let Some(prev) = self.state.get(&bike.number) {
                    if prev.station_uid != cur.uid {
                        let src_name = clean_name(
                            &self.stations.stations.get(&prev.station_uid).unwrap().name,
                        );
                        let dst_name = clean_name(&cur.name);

                        info!(
                            "{} Bike {} moved from {src_name} to {dst_name} in {} minutes",
                            NaiveDateTime::from_timestamp(prev.timestamp as i64, 0),
                            bike.number,
                            (timestamp - prev.timestamp) / 60
                        );

                        self.rides
                            .write(&RideEvent {
                                bike_id: bike.number,
                                src: RideLocation {
                                    timestamp: prev.timestamp,
                                    station_id: prev.station_uid,
                                },
                                dst: RideLocation {
                                    timestamp,
                                    station_id: cur.uid,
                                },
                            })
                            .unwrap();

                        rides += 1;
                    }
                }

                self.state.insert(
                    bike.number,
                    StateRecord {
                        timestamp: timestamp as u64,
                        station_uid: cur.uid,
                    },
                );
            }
        }
        rides
    }
}

fn clean_name(name: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\*?\(.+").unwrap();
    }

    return RE.replace(name, "").to_string().trim().to_string();
}
