use serde::Serialize;

use actix_files::NamedFile;
use actix_web::{get, web, Responder, Result};

use crate::rides::{RideEvent, RidesFilter, RidesReader};

#[derive(Serialize)]
struct RidesResponse {
    rides: Vec<RideEvent>,
    last_event_id: usize,
}

#[get("/rides.json")]
pub async fn rides(filter: web::Query<RidesFilter>) -> impl Responder {
    let filter = filter.into_inner();

    let mut rides = Vec::new();
    let mut last_event_id = 0;
    for (event_id, ride) in RidesReader::new(filter) {
        last_event_id = event_id;
        rides.push(ride);
    }

    web::Json(RidesResponse {
        rides,
        last_event_id,
    })
}

#[get("/stations.json")]
pub async fn stations() -> Result<NamedFile> {
    Ok(NamedFile::open("stations.json")?)
}
