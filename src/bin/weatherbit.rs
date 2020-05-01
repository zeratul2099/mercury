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
#[path = "../weatherbit_model.rs"]
mod weatherbit_model;
#[path = "../models.rs"]
mod models;
#[path = "../schema.rs"]
mod schema;

use self::models::insert_values;
use common::{establish_connection, get_settings};
use weatherbit_model::{WeatherbitCurrent, WeatherbitForecast};
use std::fs::File;

fn main() {
    let settings = get_settings();
    let curr_url = format!(
        "https://api.weatherbit.io/v2.0/current?key={}&lat={}&lon={}",
        settings.weatherbit_api_key, settings.lat_lon.0, settings.lat_lon.1
    );
    println!("{}", curr_url);
    let content: WeatherbitCurrent = reqwest::get(curr_url.as_str()).unwrap().json().unwrap(); //.expect("weather request failed");
    println!(
        "t: {}, h: {}, {}, wspd: {}",
        content.data.get(0).unwrap().temp,
        content.data.get(0).unwrap().rh,
        content.data.get(0).unwrap().weather.description,
        content.data.get(0).unwrap().wind_spd
    );
    println!("{}", serde_json::to_string_pretty(&content).unwrap());
    let file = File::create("weatherbitcurrdump.json").unwrap();
    serde_json::to_writer_pretty(&file, &content).unwrap();
    
    let fc_url = format!(
        "https://api.weatherbit.io/v2.0/forecast/daily?key={}&lat={}&lon={}",
        settings.weatherbit_api_key, settings.lat_lon.0, settings.lat_lon.1
    );
    println!("{}", fc_url);
    let content: WeatherbitForecast = reqwest::get(fc_url.as_str()).unwrap().json().unwrap(); //.expect("weather request failed");
    println!(
        "t: {}, h: {}, {}, wspd: {}",
        content.data.get(0).unwrap().temp,
        content.data.get(0).unwrap().rh,
        content.data.get(0).unwrap().weather.description,
        content.data.get(0).unwrap().wind_spd
    );
    println!("{}", serde_json::to_string_pretty(&content).unwrap());
    let file = File::create("weatherbitfcdump.json").unwrap();
    serde_json::to_writer_pretty(&file, &content).unwrap();
    let connection = establish_connection(&settings);
    insert_values(
        &connection,
        &settings,
        &0,
        &content.data.get(0).unwrap().temp,
        &(content.data.get(0).unwrap().rh),
    );
}
