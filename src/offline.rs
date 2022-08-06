use std::{fs::{self, OpenOptions, File}, collections::HashMap, io::{BufWriter, BufReader}, path::PathBuf, sync::{Weak, Arc}};
use indicatif::ProgressBar;

use log::{error, debug};

use crate::{Record, stations::Stations, processor::process, input::JSON, rides::Rides, logging::LoggingAwareProgressBar};

struct JsonFile {
    path: std::path::PathBuf,
    timestamp: u32,
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
            error!("Failed to list files in {}: {:?}", path.display(), x);
            None
        }
    }
}

pub fn load_from_disk(input_path: &PathBuf, output_path: &PathBuf, stations: &mut Stations) {
    let mut state = HashMap::<u32, Record>::new();
    let mut rides = Rides::new_blank(output_path);

    let files = get_files(input_path).unwrap();
    let bar = LoggingAwareProgressBar::new(files.len() as u64);
    bar.set_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {per_sec:7} (ETA {eta_precise:>5})").unwrap().progress_chars("#>-"));

    for JsonFile{timestamp, path} in files {
        debug!("Processing {path:?}");

        match File::open(&path) {
            Ok(f) => {
                let reader = BufReader::with_capacity(1024*1024*5, f);
                match serde_json::from_reader::<BufReader<File>, JSON>(reader) {
                    Ok(p) => {
                        process(timestamp, &p, &mut state, stations, &mut rides);
                    }
                    Err(e) => error!("Failed to parse json: {:?}", e),
                }
            }
            Err(e) => error!("Failed to open file: {e}")
        }

        bar.inc(1);
    }
}