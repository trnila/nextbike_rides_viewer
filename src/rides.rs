use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::processor::CsvRide;

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

    pub fn new_blank(path: &PathBuf) -> Self {
        Self::new(path, false)
    }

    pub fn new_appending(path: &PathBuf) -> Self {
        Self::new(path, true)
    }

    pub fn write(&mut self, ride: &CsvRide) -> Result<(), ()> {
        serde_json::to_writer(&mut self.writer, ride).unwrap();
        self.writer.write_all(b"\n").unwrap();

        if self.flush {
            self.writer.flush().unwrap();
        }

        Ok(())
    }
}
