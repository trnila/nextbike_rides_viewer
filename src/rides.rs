use std::{
    cmp::Ordering,
    fs::{File, OpenOptions},
    io::{BufWriter, Cursor, Read, Write},
    path::PathBuf,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use memmap::{Mmap, MmapOptions};
use serde::{Deserialize, Serialize};

// u32 bike_id + 2 * (u64 timestamp + u32 station_id)
const RECORD_SIZE: usize = 4 + 2 * (8 + 4);

#[derive(Serialize, Deserialize, Debug)]
pub struct RideEvent {
    pub bike_id: u32,

    pub src: RideLocation,
    pub dst: RideLocation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RideLocation {
    pub timestamp: u64,
    pub station_id: u32,
}

pub struct RidesWriter {
    writer: BufWriter<File>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RidesFilter {
    from: Option<u64>,
    last_event_id: Option<usize>,
    limit: Option<usize>,
}

pub struct RidesReader {
    filter: RidesFilter,
    mmap: memmap::Mmap,
    pos: usize,
    count: usize,
}

impl RidesWriter {
    pub fn new(path: PathBuf) -> Self {
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path.clone())
            .unwrap();
        RidesWriter {
            writer: BufWriter::new(f),
        }
    }

    pub fn write(&mut self, ride: &RideEvent) -> Result<(), std::io::Error> {
        self.writer.write_u32::<LittleEndian>(ride.bike_id)?;
        self.write_location(&ride.src)?;
        self.write_location(&ride.dst)?;
        self.writer.flush()?;
        Ok(())
    }

    fn write_location(&mut self, location: &RideLocation) -> Result<(), std::io::Error> {
        self.writer.write_u64::<LittleEndian>(location.timestamp)?;
        self.writer.write_u32::<LittleEndian>(location.station_id)?;
        Ok(())
    }
}

fn ride_matches(ride: &RideEvent, event_id: usize, filter: &RidesFilter) -> bool {
    if let Some(from) = filter.from {
        if ride.src.timestamp < from && ride.dst.timestamp < from {
            return false;
        }
    }

    if let Some(num) = filter.last_event_id {
        if event_id < num {
            return false;
        }
    }

    true
}

fn decode_ride<R: Read>(mut file: R) -> Result<RideEvent, std::io::Error> {
    Ok(RideEvent {
        bike_id: file.read_u32::<LittleEndian>()?,
        src: RideLocation {
            timestamp: file.read_u64::<LittleEndian>()?,
            station_id: file.read_u32::<LittleEndian>()?,
        },
        dst: RideLocation {
            timestamp: file.read_u64::<LittleEndian>()?,
            station_id: file.read_u32::<LittleEndian>()?,
        },
    })
}

fn find_first<F>(mmap: &Mmap, cmp_fn: F) -> usize
where
    F: Fn(&RideEvent) -> Ordering,
{
    let mut size = mmap.len() / RECORD_SIZE;
    let mut left = 0;
    let mut right = size;
    let mut pos = 0;
    while left < right {
        let mid = left + size / 2;

        let offset = mid * RECORD_SIZE;
        let event = decode_ride(Cursor::new(&mmap[offset..offset + RECORD_SIZE])).unwrap();

        let cmp = cmp_fn(&event);

        if cmp == Ordering::Less {
            left = mid + 1;
        } else {
            right = mid;
            pos = mid * RECORD_SIZE;
        }

        size = right - left;
    }

    return pos;
}

impl RidesReader {
    pub fn new(filter: RidesFilter) -> Self {
        let file = File::open("rides.bin").unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        let mut pos = 0;

        if let Some(ts) = filter.from {
            pos = find_first(&mmap, |ride| {
                if ride.src.timestamp < ts {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            });
        }

        if let Some(last_event_id) = filter.last_event_id {
            pos = last_event_id * RECORD_SIZE;
        }

        RidesReader {
            filter,
            mmap,
            pos,
            count: 0,
        }
    }
}

impl Iterator for RidesReader {
    type Item = (usize, RideEvent);

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.filter.limit.unwrap_or(100) {
            return None;
        }

        loop {
            self.pos += RECORD_SIZE;
            if self.pos >= self.mmap.len() {
                return None;
            }

            match decode_ride(Cursor::new(&self.mmap[self.pos..self.pos + RECORD_SIZE])) {
                Err(_err) => return None,
                Ok(ride) => {
                    let event_id = self.pos / RECORD_SIZE - 1;
                    if ride_matches(&ride, event_id, &self.filter) {
                        self.count += 1;
                        return Some((event_id, ride));
                    }
                }
            }
        }
    }
}
