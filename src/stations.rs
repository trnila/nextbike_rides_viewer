use std::{collections::{HashMap, hash_map::Entry}, fs::{File, OpenOptions}, io::BufReader};

use serde::{Serialize, Deserialize};

use crate::input::StationId;

#[derive(Serialize, Deserialize, Debug)]
pub struct Station {
    pub name: String,
    pub lat: f32,
    pub lng: f32,
}

pub struct Stations {
    pub stations: HashMap::<StationId, Station>,
    path: String,
}

impl Stations {
    pub fn new(path: &str) -> Stations {
        let stations: HashMap::<StationId, Station> = match File::open(path) {
            Ok(f) => serde_json::from_reader(BufReader::new(f)).unwrap(),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => HashMap::<StationId, Station>::new(),
                _ => panic!(),
            }
        };

        Stations {
            stations,
            path: path.to_string(),
        }
    }

    pub fn add_station(&mut self, id: StationId, name: &str, lat: f32, lng: f32) {
        match self.stations.entry(id) {
            Entry::Vacant(v) => {
                v.insert(Station{
                    name: name.to_string(),
                    lat,
                    lng
                });

                let w = OpenOptions::new().create(true).write(true).open(&self.path).unwrap();
                serde_json::to_writer_pretty(w, &self.stations).unwrap();
            }
            Entry::Occupied(_) => (),
        }
    }
}