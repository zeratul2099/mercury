extern crate diesel;
#[macro_use] extern crate lazy_static;
extern crate reqwest;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[path = "../common.rs"]
mod common;

use common::{get_settings,establish_connection};


#[derive(Deserialize)]
struct WeatherData {
    latitude: f64,
    longitude: f64,
    timezone: String,
    offset: i32,
    currently: Currently,

}

#[derive(Deserialize)]
struct Currently {
    time: i32,
    summary: String,
    icon: String,
    precipIntensity: i32,
    precipProbability: i32,
    temperature: f64,
    apparentTemperature: f64,
    dewPoint: f64,
    humidity: f64,
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
    println!("Hello World! {}", settings.darksky_api_key);
    let url = format!("https://api.darksky.net/forecast/{}/{},{}?exclude=minutely,hourly,daily,alerts,flags&units=si", settings.darksky_api_key, settings.lat_lon.0, settings.lat_lon.1);
    println!("{:?}", url);
    let content: WeatherData = reqwest::get(url.as_str()).unwrap().json().unwrap(); //.expect("weather request failed");
    println!("t: {}, h: {}, {}, wspd: {}", content.currently.temperature, content.currently.humidity * 100.0, content.currently.summary, content.currently.windSpeed);
}

