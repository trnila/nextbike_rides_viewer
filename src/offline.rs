use std::{fs::{self, OpenOptions, File}, collections::HashMap, io::{BufWriter, BufReader}};

use log::{error, debug};

use crate::{Record, stations::Stations, processor::process, input::JSON};

struct JsonFile {
    path: std::path::PathBuf,
    timestamp: u32,
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

pub fn load_from_disk(path: &str) {
    let mut state = HashMap::<u32, Record>::new();
    let mut s = Stations::new("viewer/public/stations.json");

    let f = OpenOptions::new().write(true).create(true).truncate(true).open("viewer/public/rides.json").unwrap();
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