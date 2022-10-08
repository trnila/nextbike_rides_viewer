use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RideEvent {
    pub bike_id: u32,

    pub src: RideLocation,
    pub dst: RideLocation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RideLocation {
    pub timestamp: u64,
    pub name: String,
    pub lat: f32,
    pub lng: f32,
}

pub struct Rides {
    writer: BufWriter<File>,
    flush: bool,
}

impl Rides {
    pub fn new(path: &PathBuf, append: bool) -> Self {
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(path)
            .unwrap();
        Rides {
            writer: BufWriter::new(f),
            flush: append,
        }
    }

    pub fn new_appending(path: &PathBuf) -> Self {
        Self::new(path, true)
    }

    pub fn write(&mut self, ride: &RideEvent) -> Result<(), ()> {
        serde_json::to_writer(&mut self.writer, ride).unwrap();
        self.writer.write_all(b"\n").unwrap();

        if self.flush {
            self.writer.flush().unwrap();
        }

        Ok(())
    }
}
