use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::io;
use std::io::LineWriter;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::HashSet;


#[derive(Serialize, Deserialize, Debug)]
struct Bike {
    number: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Place {
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
    timestamp: u32,
}

struct JsonFile {
    path: std::path::PathBuf,
    timestamp: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Position {
    lat: f32,
    lng: f32,
}

fn main() {
    println!("Hello, world!");

    let mut state = HashMap::<u32, Record>::new();

    let mut paths: Vec<JsonFile> = fs::read_dir("data").unwrap().map(|p| {
        let path = p.unwrap().path();
        JsonFile{
            path: path.clone(),
            timestamp: path.file_stem().unwrap().to_str().unwrap().parse::<u32>().unwrap(),
        }
    }).collect();

    paths.sort_by_key(|p| p.timestamp);

    let mut stations = HashMap::<String, Position>::new();


    let mut buf = [0u8; 5 * 1024 * 1024];
    for JsonFile{timestamp, path} in paths {
        //let ppp = path.unwrap();
        //let ts = ppp.path().file_stem().unwrap().to_str().unwrap().parse::<u32>().unwrap();

        //let data = fs::read_to_string(&path.unwrap().path()).expect("Unable to read file");
        let f = File::open(&path).unwrap();
        //let mut data = String::new();
        //f.read_to_string(&mut data);
        
        //f.read(&mut buf);

        let reader = BufReader::with_capacity(1024*1024*5, f);
        match serde_json::from_reader::<BufReader<File>, JSON>(reader) {
            Ok(p) => {
                if p.countries.len() != 1 {
                    continue
                }

                for place in &p.countries[0].cities[0].places {
                    if place.name.starts_with("BIKE") {
                        continue
                    }

                    stations.insert(place.name.clone(), Position { lat: place.lat, lng: place.lng });

                    for bike in &place.bike_list {
                        //println!("x{:?}", bike.number.parse::<i32>().unwrap());
                        let id = bike.number.parse::<u32>().unwrap();

                        if let Some(rec) = state.get(&id) {
                            if rec.station != place.name {
                                //println!("{:?} moved to {:?} {}", rec, place.name, timestamp - rec.timestamp);
                                println!(
                                    "{}, {}, {}, {}, {}, {}, {}, {}, {}",
                                    id, rec.timestamp, rec.station.replace(',', " "), timestamp, place.name.replace(',', " "),
                                    stations.get(&rec.station).unwrap().lat, stations.get(&rec.station).unwrap().lng,
                                    place.lat, place.lng
                                );
                            }
                        }

                        state.insert(id, Record{
                            timestamp: timestamp,
                            station: place.name.clone(),
                        });
                    }
                }
            }
            Err(e) => println!("errr: {:?}", e),
        }


        //let p: JSON = serde_json::from_str(&data).unwrap();
        //println!("{:?}", p)

    }

    let mut w = LineWriter::new(File::create("stations.csv").unwrap());
    for (name, pos) in stations.into_iter() {
        writeln!(w, "{}, {}, {}", name.replace(',', " "), pos.lat, pos.lng);
    }

//    println!("{:?}", stations)
}
