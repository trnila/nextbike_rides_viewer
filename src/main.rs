use actix_web::get;
use actix_web::HttpResponse;
use actix_web::Responder;
use log::{error, info};

use std::path::PathBuf;
use std::thread;
use std::time;
use std::time::Duration;
use std::time::SystemTime;

use actix_web::{middleware::Logger, App};
use nextbike::input::JsonResponse;
use nextbike::processor::RidesProcessor;
use nextbike::rides::RidesWriter;
use nextbike::stations::Stations;

use actix_web::HttpServer;

fn start_scrapper(interval: Duration) {
    thread::spawn(move || {
        let stations = Stations::new("stations.json".into());
        let mut processor =
            RidesProcessor::new(stations, RidesWriter::new(PathBuf::from("rides.bin")));

        loop {
            scrap_data(&mut processor);
            thread::sleep(interval);
        }
    });
}

fn scrap_data(processor: &mut RidesProcessor) {
    let ts = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    info!("Downloading new data at {ts}");
    match reqwest::blocking::get("https://api.nextbike.net/maps/nextbike-live.json?city=271") {
        Ok(resp) => match resp.text() {
            Ok(body) => match serde_json::from_str::<JsonResponse>(&body) {
                Ok(json) => {
                    let rides = processor.process(ts, &json);
                    info!("{rides} new rides found");
                }
                Err(err) => error!("Failed to parse json: {err}"),
            },
            Err(err) => error!("Failed to parse response: {err}"),
        },
        Err(err) => error!("Failed to fetch: {err}"),
    }
}

static INDEX_HTML: &str = include_str!("../viewer/dist/index.html");

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_HTML)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    start_scrapper(Duration::from_secs(60));

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(nextbike::api::rides)
            .service(nextbike::api::stations)
            .wrap(Logger::new("%U %s %D"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
