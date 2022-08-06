use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use regex::Regex;
use log::{debug, error};


#[derive(Serialize, Deserialize, Debug)]
struct Bike {
    number: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Place {
    uid: StationId,
    lat: f32,
    lng: f32,
    name: String,
    bike_list: Vec<Bike>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cities {
    name: String,
    places: Vec<Place>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Countries {
    cities: Vec<Cities>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JSON {
    countries: Vec<Countries>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    station: String,
    station_uid: StationId,
    timestamp: u64,
}

struct JsonFile {
    path: std::path::PathBuf,
    timestamp: u32,
}

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
struct Station {
    name: String,
    lat: f32,
    lng: f32,
}

type StationId = u32;

struct Stations {
    stations: HashMap::<StationId, Station>,
    path: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Locatin {
    timestamp: u64,
    name: String,
    lat: f32,
    lng: f32,
}


#[derive(Serialize, Deserialize, Debug)]
struct CsvRide {
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


fn process(timestamp: u32, p: &JSON, state: &mut HashMap::<u32, Record>, stations: &mut Stations, w: &mut BufWriter<File>) {
    if p.countries.len() != 1 {
        return;
    }

    for place in &p.countries[0].cities[0].places {
        if place.name.starts_with("BIKE") {
            continue
        }

        stations.add_station(place.uid, &place.name, place.lat, place.lng);

        for bike in &place.bike_list {
            //println!("x{:?}", bike.number.parse::<i32>().unwrap());
            let id = bike.number.parse::<u32>().unwrap();

            if let Some(rec) = state.get(&id) {
                if rec.station_uid != place.uid {
                    let s = stations.stations.get(&rec.station_uid).unwrap();

                    serde_json::to_writer(&mut *w, &CsvRide{
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

                    w.write_all(b"\n").unwrap();
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

fn get_files(path: &str) -> Option<Vec<JsonFile>> {
    match fs::read_dir(path) {
        Ok(files_iter) => {
            let mut files: Vec<JsonFile> = files_iter.filter_map(|path| {
                match path {
                    Ok(path) => {
                        match path.path().file_stem() {
                            Some(stem) =>{
                                Some(JsonFile{
                                    path: path.path().clone(),
                                    timestamp: stem.to_str().unwrap().parse::<u32>().unwrap(),
                                })
                            }
                            None => {
                                error!("Stem for path {path:?} not found.");
                                None
                            }
                        }
                    }
                    Err(_e) => unreachable!()
                }                
            }).collect();
            files.sort_by_key(|p| p.timestamp);
            Some(files)
        }
        Err(x) => {
            error!("Failed to list files in {}: {:?}", path, x);
            None
        }
    }
}

impl Stations {
    fn new(path: &str) -> Stations {
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

    fn add_station(&mut self, id: StationId, name: &str, lat: f32, lng: f32) {
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


fn load_from_disk(path: &str) {
    let mut state = HashMap::<u32, Record>::new();
    let mut s = Stations::new("viewer/public/stations.json");

    let f = OpenOptions::new().write(true).create(true).open("viewer/public/rides.json").unwrap();
    let mut w = BufWriter::new(f);

    for JsonFile{timestamp, path} in get_files(path).unwrap() {
        debug!("Processing {path:?}");

        match File::open(&path) {
            Ok(f) => {
                let reader = BufReader::with_capacity(1024*1024*5, f);
                match serde_json::from_reader::<BufReader<File>, JSON>(reader) {
                    Ok(p) => {
                        process(timestamp, &p, &mut state, &mut s, &mut w);
                    }
                    Err(e) => error!("Failed to parse json: {:?}", e),
                }
            }
            Err(e) => error!("Failed to open file: {e}")
        }
    }
}

fn main() {
    env_logger::init();

    load_from_disk("data");

//    let body = reqwest::blocking::get("https://api.nextbike.net/maps/nextbike-live.json?city=271").unwrap()
    //.text().unwrap();
    /*
    println!("body = {:?}", body);
    match serde_json::from_str(&body) {
        Ok(p) => {
            process(SystemTime::now().elapsed().unwrap().as_secs().try_into().unwrap(), &p, &mut state, &mut stations);
        }
        Err(e) => println!("errr: {:?}", e)
    }
    */
}
