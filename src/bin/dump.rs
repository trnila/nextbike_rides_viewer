use std::path::PathBuf;

use chrono::{DateTime, Local, TimeZone};
use clap::Parser;
use nextbike::rides::{RidesFilter, RidesReader};
use nextbike::stations::Stations;

#[derive(Debug, Parser)]
struct DumpArgs {
    #[arg(long, value_parser = parse_from_timestamp)]
    from: Option<u64>,
    #[arg(long)]
    last_event_id: Option<usize>,
    #[arg(long, default_value_t = usize::MAX)]
    limit: usize,
}

fn parse_from_timestamp(input: &str) -> Result<u64, String> {
    if let Ok(ts) = input.parse::<u64>() {
        return Ok(ts);
    }

    let datetime_formats = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y-%m-%d"];

    for fmt in datetime_formats {
        if let Ok(dt) = Local.datetime_from_str(input, fmt) {
            return Ok(dt.timestamp() as u64);
        }

        if fmt == "%Y-%m-%d" {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(input, fmt) {
                let dt = date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| "Invalid time value".to_string())?;
                let local_dt = Local
                    .from_local_datetime(&dt)
                    .single()
                    .ok_or_else(|| "Ambiguous local date/time".to_string())?;
                return Ok(local_dt.timestamp() as u64);
            }
        }
    }

    Err(
        "Invalid --from value. Use unix timestamp or date/time like '2026-05-26 14:30:00' or '2026-05-26'"
            .to_string(),
    )
}

fn format_timestamp(ts: u64) -> String {
    let dt: DateTime<Local> = Local.timestamp_opt(ts as i64, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn format_duration(start: u64, end: u64) -> String {
    assert!(start <= end);
    let duration_secs = end - start;
    let hours = duration_secs / 3600;
    let minutes = (duration_secs % 3600) / 60;
    let seconds = duration_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn main() {
    let args = DumpArgs::parse();
    let stations = Stations::new(PathBuf::from("stations.json"));
    let rides = RidesReader::new(RidesFilter::new(
        args.from,
        args.last_event_id,
        Some(args.limit),
    ));

    for (event_id, ride) in rides {
        let src_station = stations
            .stations
            .get(&ride.src.station_id)
            .map(|s| s.name.as_str())
            .unwrap_or("Unknown");
        let dst_station = stations
            .stations
            .get(&ride.dst.station_id)
            .map(|s| s.name.as_str())
            .unwrap_or("Unknown");

        let duration = format_duration(ride.src.timestamp, ride.dst.timestamp);

        println!(
            "{event_id:<10} {:<10} {duration} {} ({}) -> {} ({})",
            ride.bike_id,
            src_station,
            format_timestamp(ride.src.timestamp),
            dst_station,
            format_timestamp(ride.dst.timestamp),
        );
    }
}
