use std::{fs::{self, OpenOptions, File}, collections::HashMap, io::{BufWriter, BufReader}, path::PathBuf, sync::{Weak, Arc}};
use indicatif::ProgressBar;

use log::{error, debug};

use crate::{Record, stations::Stations, input::JSON, rides::Rides, logging::LoggingAwareProgressBar, processor::RidesProcessor};

struct JsonFile {
    path: std::path::PathBuf,
    timestamp: u64,
}

fn get_files(path: &PathBuf) -> Option<Vec<JsonFile>> {
    match fs::read_dir(path) {
        Ok(files_iter) => {
            let mut files: Vec<JsonFile> = files_iter.filter_map(|path| {
                match path {
                    Ok(path) => {
                        match path.path().file_stem() {
                            Some(stem) =>{
                                Some(JsonFile{
                                    path: path.path().clone(),
                                    timestamp: stem.to_str().unwrap().parse::<u64>().unwrap(),
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
            error!("Failed to list files in {}: {:?}", path.display(), x);
            None
        }
    }
}

pub fn load_from_disk(input_path: &PathBuf, output_path: &PathBuf, stations: Stations) {
    let mut processor = RidesProcessor::new(stations, Rides::new_blank(output_path));

    let files = get_files(input_path).unwrap();
    let bar = LoggingAwareProgressBar::new(files.len() as u64);
    bar.set_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {msg:>10} {per_sec:>10} files (ETA {eta_precise:>5})").unwrap().progress_chars("#>-"));

    let mut total_rides = 0u64;
    for JsonFile{timestamp, path} in files {
        debug!("Processing {path:?}");

        match std::fs::read_to_string(&path) {
            Ok(str) => {
                match serde_json::from_str::<JSON>(&str) {
                    Ok(json) => {
                        total_rides += processor.process(timestamp, &json);
                        bar.set_message(format!("{total_rides} rides"));
                    }
                    Err(e) => error!("Failed to parse JSON {path:?}: {e:?}"),
                }
            }
            Err(e) => error!("Failed to open file: {e}")
        }

        bar.inc(1);
    }
}
