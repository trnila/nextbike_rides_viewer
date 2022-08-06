use std::{collections::HashMap, io::BufWriter, fs::File};
use std::io::Write;

use log::error;
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


pub fn process(timestamp: u32, p: &JSON, state: &mut HashMap::<u32, Record>, stations: &mut Stations, rides: &mut Rides) {
    if p.countries.len() != 1 {
        error!("Number of countries in {timestamp} is not 1, but {}", p.countries.len());
        return;
    }

    for place in &p.countries[0].cities[0].places {
        if place.name.starts_with("BIKE") {
            continue
        }

        stations.add_station(place.uid, &place.name, place.lat, place.lng);

        for bike in &place.bike_list {
            let id = bike.number.parse::<u32>().unwrap();

            if let Some(rec) = state.get(&id) {
                if rec.station_uid != place.uid {
                    let s = stations.stations.get(&rec.station_uid).unwrap();

                    rides.write(&CsvRide{
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
                }
            }

            state.insert(id, Record{
                timestamp: timestamp as u64,
                station: place.name.clone(),
                station_uid: place.uid,
            });
        }
    }
}

fn clean_name(name: &str) -> String {
    let re = Regex::new(r"\*?\(.+").unwrap();
    return re.replace(name, "").to_string().trim().to_string();
}