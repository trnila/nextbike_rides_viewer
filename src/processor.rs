use std::collections::HashMap;
use std::time::Duration;

use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use log::{error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::input::{BikeId, StationId};
use crate::rides::{RideEvent, RideLocation, Rides};
use crate::{input::JsonResponse, stations::Stations};

#[derive(Serialize, Deserialize, Debug)]
struct StateRecord {
    station_uid: StationId,
    timestamp: u64,
}

pub struct RidesProcessor {
    stations: Stations,
    state: HashMap<BikeId, StateRecord>,
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
        for place in &json.countries[0].cities[0].places {
            if place.name.starts_with("BIKE") {
                continue;
            }

            self.stations
                .add_station(place.uid, &place.name, place.lat, place.lng);

            for bike in &place.bike_list {
                if let Some(rec) = self.state.get(&bike.number) {
                    if rec.station_uid != place.uid {
                        let s = self.stations.stations.get(&rec.station_uid).unwrap();

                        let src_name =
                            clean_name(&self.stations.stations.get(&rec.station_uid).unwrap().name);
                        let dst_name = clean_name(&place.name);

                        info!(
                            "{} Bike {} moved from {src_name} to {dst_name} in {} minutes",
                            NaiveDateTime::from_timestamp(rec.timestamp as i64, 0),
                            bike.number,
                            (timestamp - rec.timestamp) / 60
                        );

                        self.rides
                            .write(&RideEvent {
                                bike_id: bike.number,
                                src: RideLocation {
                                    timestamp: rec.timestamp,
                                    name: src_name,
                                    lat: s.lat,
                                    lng: s.lng,
                                },
                                dst: RideLocation {
                                    timestamp,
                                    name: dst_name,
                                    lat: place.lat,
                                    lng: place.lng,
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
                        station_uid: place.uid,
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
