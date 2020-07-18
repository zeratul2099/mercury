//#![feature(plugin, custom_derive)]
#![allow(proc_macro_derive_resolution_fallback, dead_code)]
#[macro_use]
extern crate lazy_static;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate itertools;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate chrono_tz;
extern crate time;

#[path = "../common.rs"]
mod common;
#[path = "../models.rs"]
mod models;
#[path = "../schema.rs"]
mod schema;

use self::models::insert_values;
use crate::common::{establish_connection, get_settings, WeatherData};
use std::fs::File;

fn main() {
    let settings = get_settings();
    let url = format!(
        "https://api.darksky.net/forecast/{}/{},{}?exclude=minutely,alerts,flags&units=si",
        settings.darksky_api_key, settings.lat_lon.0, settings.lat_lon.1
    );
    let content: WeatherData = reqwest::get(url.as_str()).unwrap().json().unwrap(); //.expect("weather request failed");
    println!(
        "t: {}, h: {}, {}, wspd: {}",
        content.currently.temperature,
        content.currently.humidity * 100.0,
        content.currently.summary,
        content.currently.windSpeed
    );
    println!("{}", serde_json::to_string_pretty(&content).unwrap());
    let file = File::create("weatherdump.json").unwrap();
    serde_json::to_writer_pretty(&file, &content).unwrap();
    let settings = get_settings();
    let connection = establish_connection(&settings);
    insert_values(
        &connection,
        &settings,
        &0,
        &content.currently.temperature,
        &(content.currently.humidity * 100.0),
    );
}
