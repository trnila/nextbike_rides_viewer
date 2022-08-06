use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::thread;
use std::time;
use std::time::SystemTime;
use regex::Regex;
use log::{debug, error};

mod input;
mod offline;
mod processor;
mod stations;

use offline::*;
use input::*;
use processor::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    station: String,
    station_uid: StationId,
    timestamp: u64,
}

fn online() {
    loop {
        println!("cus");
        thread::sleep(time::Duration::from_millis(1000));
    }
}

fn download() -> Result<JSON, Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get("https://api.nextbike.net/maps/nextbike-live.json?city=271")?.text()?;
    Ok(serde_json::from_str::<JSON>(&body)?)
}

fn main() {
    env_logger::init();

    let mut state = HashMap::<u32, Record>::new();
    let mut s = stations::Stations::new("viewer/public/stations.json");

    let f = OpenOptions::new().create(true).append(true).open("viewer/public/rides.json").unwrap();
    let mut w = BufWriter::new(f);

    //online();

    load_from_disk("data");

    let p = download().unwrap();

    process(SystemTime::now().elapsed().unwrap().as_secs().try_into().unwrap(), &p, &mut state, &mut s, &mut w);
}
