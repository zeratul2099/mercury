//#![feature(plugin, custom_derive)]
#![allow(proc_macro_derive_resolution_fallback,dead_code)]
#[macro_use] extern crate lazy_static;
extern crate reqwest;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate itertools;
#[macro_use] extern crate diesel;
extern crate chrono;

#[path = "../common.rs"]
mod common;
#[path = "../models.rs"]
mod models;
#[path = "../schema.rs"]
mod schema;


use std::fs::File;
use common::{get_settings,establish_connection};
use self::models::{insert_values};

#[derive(Deserialize, Serialize, Debug)]
struct WeatherData {
    latitude: f64,
    longitude: f64,
    timezone: String,
    offset: i32,
    currently: Currently,

}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct Currently {
    time: i32,
    summary: String,
    icon: String,
    precipIntensity: i32,
    precipProbability: i32,
    temperature: f32,
    apparentTemperature: f64,
    dewPoint: f64,
    humidity: f32,
    pressure: f64,
    windSpeed: f64,
    windGust: f64,
    windBearing: i32,
    cloudCover: f64,
    uvIndex: i32,
    visibility: f64,
    ozone: f64,
}

fn main() {
    let settings = get_settings();
    let url = format!("https://api.darksky.net/forecast/{}/{},{}?exclude=minutely,hourly,daily,alerts,flags&units=si", settings.darksky_api_key, settings.lat_lon.0, settings.lat_lon.1);
    let content: WeatherData = reqwest::get(url.as_str()).unwrap().json().unwrap(); //.expect("weather request failed");
    println!("t: {}, h: {}, {}, wspd: {}", content.currently.temperature, content.currently.humidity * 100.0, content.currently.summary, content.currently.windSpeed);
    println!("{}", serde_json::to_string_pretty(&content).unwrap());
    let file = File::create("weatherdump.json").unwrap();
    serde_json::to_writer_pretty(&file, &content).unwrap();
    let settings = get_settings();
    let connection = establish_connection(&settings);
    insert_values(&connection, &settings, &0, &content.currently.temperature, &(content.currently.humidity * 100.0));
}

