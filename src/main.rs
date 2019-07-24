#![allow(proc_macro_derive_resolution_fallback, dead_code)]
#![feature(plugin, proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate itertools;
extern crate rocket_contrib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate diesel;
extern crate chrono;
extern crate chrono_tz;
extern crate tera;
extern crate time;

pub mod common;
pub mod models;
pub mod schema;

use self::models::*;
use chrono::prelude::*;
use chrono_tz::Tz;
use common::{check_notification, establish_connection, get_settings, WeatherData};
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tera::{from_value, to_value, Error, GlobalFn, Value};

#[derive(FromForm)]
struct SendQuery {
    id: i32,
    t: f32,
    h: f32,
}

#[derive(FromForm)]
struct PlotQuery {
    old: bool,
}

fn main() {
    let settings = get_settings();
    let timezone: Tz = settings.timezone.parse().unwrap();
    rocket::ignite()
        .mount("/", routes![send,latest,history,files,simple,plots,oldplots,weather,gauges,table,render_table,mean,single_mean])
//          .attach(Template::fairing())
        .attach(Template::custom(move |engines| {
            engines.tera.register_function("convert_tz", make_convert_tz(timezone));
        }))
        .launch();
}

fn make_convert_tz(timezone: Tz) -> GlobalFn {
    Box::new(move |args| -> Result<Value, Error> {
        match args.get("datetime") {
            Some(val) => match from_value::<i64>(val.clone()) {
                Ok(v) => match args.get("format") {
                    Some(f) => Ok(to_value(
                        Utc.timestamp(v, 0)
                            .with_timezone(&timezone)
                            .format(f.as_str().unwrap())
                            .to_string(),
                    ).unwrap()),
                    None => Ok(
                        to_value(Utc.timestamp(v, 0).with_timezone(&timezone).to_string()).unwrap(),
                    ),
                },
                Err(e) => Err(format!("oops error converting timezone: {}: {}", val, e).into()),
            },
            None => Err("oops no datetime given".into()),
        }
    })
}

#[get("/simple")]
fn simple() -> Template {
    let mut context = HashMap::new();
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings, 1, None);
    context.insert("latest_values".to_string(), values);
    Template::render("simple", context)
}

#[get("/plots")]
fn plots() -> Template {
    let context = HashMap::<String, String>::new();
    Template::render("plots", context)
}

#[get("/oldplots")]
fn oldplots() -> Template {
    let context = HashMap::<String, String>::new();
    Template::render("plots_old", context)
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
}

#[get("/weather")]
fn weather() -> Template {
    let mut file = File::open("weatherdump.json").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let conditions: WeatherData = serde_json::from_str(&buf).unwrap();
    let context = WeatherContext {
        conditions: conditions,
    };
    Template::render("weather", context)
}

#[get("/table")]
fn table() -> Template {
    render_table(None)
}

#[get("/table/<s_id>")]
fn render_table(s_id: Option<i32>) -> Template {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings, 100, s_id);
    let mut context = HashMap::new();
    context.insert("result".to_string(), values);
    Template::render("table", context)
}

#[get("/api/send?<query..>")]
fn send(query: Form<SendQuery>) -> String {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    // HACK to fix decimal formatting errors of arduino implementation
    let temp: f32;
    let hum: f32;
    if query.t * 100.0 % 10.0 == 9.0 {
        temp = (query.t * 10.0).round() / 10.0;
    } else {
        temp = query.t;
    }
    if query.h * 100.0 % 10.0 == 9.0 {
        hum = (query.h * 10.0).round() / 10.0;
    } else {
        hum = query.h;
    }
    insert_values(&connection, &settings, &query.id, &temp, &hum);
    check_notification(&settings, query.id as i64, &"t".to_string(), query.t as f64);
    check_notification(&settings, query.id as i64, &"h".to_string(), query.h as f64);
    "OK".to_string()
}

#[get("/api/latest")]
fn latest() -> Json<Vec<(String, String, String, f32, f32, bool)>> {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings, 1, None);
    Json(values)
}

#[get("/api/history/<days>")]
fn history(days: i64) -> Json<Vec<(i32, String, Vec<(String, f32, f32)>)>> {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_history(&connection, &settings, days);
    Json(values)
}

#[get("/api/single_mean/<s_id>/<date>")]
fn single_mean(s_id: i32, date: String) -> Json<(f32, f32, f32, f32, f32, f32)> {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let date = format!("{}T00:00:00Z", date);
    let date = DateTime::parse_from_rfc3339(&date).unwrap().naive_utc();
    let values = get_day_mean_min_max_values(&connection, &s_id, date);
    Json(values)
}

#[get("/api/mean/<begin>/<end>")]
fn mean(begin: String, end: String) -> Json<Vec<(i32, String, Vec<(NaiveDateTime, f32, f32, f32, f32, f32, f32)>)>> {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let begin = format!("{}T00:00:00Z", begin);
    let begin = DateTime::parse_from_rfc3339(&begin).unwrap().naive_utc();
    let end = format!("{}T00:00:00Z", end);
    let end = DateTime::parse_from_rfc3339(&end).unwrap().naive_utc();
    let values = get_timespan_mean_min_max_values(&connection, &settings, begin, end);
    Json(values)
}

#[get("/static/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
