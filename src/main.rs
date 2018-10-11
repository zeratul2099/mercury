#![allow(proc_macro_derive_resolution_fallback,dead_code)]
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;
extern crate itertools;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate chrono_tz;
extern crate time;

pub mod common;
pub mod schema;
pub mod models;

use std::collections::{HashMap};
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::File;
use self::models::*;
use rocket_contrib::Json;
use rocket::response::NamedFile;
use rocket_contrib::Template;
use chrono::prelude::*;
use chrono_tz::Tz;
use common::{get_settings,check_notification,establish_connection,WeatherData};



#[derive(FromForm)]
struct SendQuery {
    id: i32,
    t: f32,
    h: f32,
}

fn main() {
    rocket::ignite().mount("/", routes![send,latest,history,files,simple,plots,weather,gauges]).attach(Template::fairing()).launch();
}

#[get("/simple")]
fn simple() -> Template {
    let mut context = HashMap::new();
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings);
    context.insert("latest_values".to_string(), values);
    Template::render("simple", context)
}

#[get("/plots")]
fn plots() -> Template {
    let context = HashMap::<String, String>::new();
    Template::render("plots", context)
}

#[get("/gauges")]
fn gauges() -> Template {
    let settings = get_settings();
    let mut context = HashMap::new();
    context.insert("num_sensors".to_string(), settings.sensor_map.len());
    Template::render("gauges", context)
}

#[derive(Deserialize, Serialize, Debug)]
struct WeatherContext {
    conditions: WeatherData,
    timestamp: String,
}

#[get("/weather")]
fn weather() -> Template {
    let settings = get_settings();
    let mut file = File::open("weatherdump.json").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let conditions: WeatherData = serde_json::from_str(&buf).unwrap();
    let ts = conditions.currently.time;
    let ts = Utc.timestamp(ts as i64, 0);
    let tz: Tz = settings.timezone.parse().unwrap();
    let ts = ts.with_timezone(&tz).to_string();
    let context = WeatherContext {
        conditions: conditions,
        timestamp: ts,
    };
    Template::render("weather", context)
}
#[get("/api/send?<query>")]
fn send(query: SendQuery) -> &'static str {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    insert_values(&connection, &settings, &query.id, &query.t, &query.h);
    check_notification(&settings, query.id as i64, &"t".to_string(), query.t as f64);
    check_notification(&settings, query.id as i64, &"h".to_string(), query.h as f64);
    "OK"
}

#[get("/api/latest")]
fn latest() -> Json<Vec<(String, String, String, f32, f32, bool)>> {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings);
    Json(values)
}

#[get("/api/history")]
fn history() -> Json<Vec<(i32, String, Vec<(String, f32, f32)>)>> {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_history(&connection, &settings);
    Json(values)
}

#[get("/static/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
