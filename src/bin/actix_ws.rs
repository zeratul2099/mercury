//#![feature(plugin, custom_derive)]
#![allow(proc_macro_derive_resolution_fallback, dead_code)]
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

#[path = "../common.rs"]
mod common;
#[path = "../models.rs"]
pub mod models;
#[path = "../schema.rs"]
pub mod schema;
#[path = "../weatherbit_model.rs"]
pub mod weatherbit_model;
use self::models::*;
use actix_files::Files;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware, error, Error, Result};
use chrono::prelude::*;
use chrono_tz::Tz;
use std::fs::File;
use std::io::prelude::*;
use tera::{Tera, GlobalFn, Value, from_value, to_value, Error as TeraError};
use crate::common::{check_notification,establish_connection, get_settings};
use crate::models::get_latest_values;
use crate::weatherbit_model::{WeatherbitCurrent,WeatherbitForecast};

fn make_convert_tz(timezone: Tz) -> GlobalFn {
    Box::new(move |args| -> Result<Value, TeraError> {
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

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/simple")]
async fn simple(
    tmpl: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings, 1, None);
    context.insert("latest_values", &values);
    let s = tmpl.render("simple.tera", &context)
        .map_err(|_| error::ErrorInternalServerError("TemplateError"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/plots")]
async fn plots(
    tmpl: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let s = tmpl.render("plots.tera", &tera::Context::new())
        .map_err(|_| error::ErrorInternalServerError("TemplateError"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/gauges")]
async fn gauges(
    tmpl: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    let settings = get_settings();
    context.insert("num_sensors", &settings.sensor_map.len());
    let s = tmpl.render("gauges.tera", &context)
        .map_err(|_| error::ErrorInternalServerError("TemplateError"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[derive(Deserialize, Serialize, Debug)]
struct WeatherbitContext {
    current: WeatherbitCurrent,
    forecast: WeatherbitForecast,
}

#[get("/weather")]
async fn weatherbit(
    tmpl: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let mut file = File::open("weatherbitcurrdump.json").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let current: WeatherbitCurrent = serde_json::from_str(&buf).unwrap();
    let mut file = File::open("weatherbitfcdump.json").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let forecast: WeatherbitForecast = serde_json::from_str(&buf).unwrap();
    let context = WeatherbitContext {
        current: current,
        forecast: forecast,
    };
    let s = tmpl.render("weatherbit.tera", &context).unwrap();
//        .map_err(|_| error::ErrorInternalServerError("TemplateError"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/table/{s_id}")]
async fn table(
    path: web::Path<(String,)>,
    tmpl: web::Data<tera::Tera>,
) -> Result<HttpResponse, Error> {
    let settings = get_settings();
    let connection = establish_connection(&settings);

    let s_id: Option<i32> = path.0.parse::<i32>().ok();
    //match Some(path.0.parse::<i32>()) {
    //    Ok(s_id) => s_id,
    //    Err(e) => None,
    //}

    let values = get_latest_values(&connection, &settings, 100, s_id);
    let mut context = tera::Context::new();
    context.insert("result", &values);


    let s = tmpl.render("table.tera", &context)
        .map_err(|_| error::ErrorInternalServerError("TemplateError"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[get("/api/latest")]
async fn latest() -> HttpResponse { 
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings, 1, None);
    HttpResponse::Ok().json(values)
}

#[get("/api/history/{days}")]
async fn history(
        path: web::Path<(i64,)>,
) -> HttpResponse {

    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_history(&connection, &settings, path.0);
    HttpResponse::Ok().json(values)
}

#[get("/api/single_mean/{s_id}/{date}")]
async fn single_mean(
        path: web::Path<(i32, String,)>,
) -> HttpResponse {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let date = format!("{}T00:00:00Z", path.1);
    let date = DateTime::parse_from_rfc3339(&date).unwrap().naive_utc();
    let values = get_day_mean_min_max_values(&connection, &path.0, date);
    HttpResponse::Ok().json(values)
}

#[get("/api/mean/{begin}/{end}")]
async fn mean(
        path: web::Path<(String, String,)>,
) -> HttpResponse {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let begin = format!("{}T00:00:00Z", path.0);
    let begin = DateTime::parse_from_rfc3339(&begin).unwrap().naive_utc();
    let end = format!("{}T00:00:00Z", path.1);
    let end = DateTime::parse_from_rfc3339(&end).unwrap().naive_utc();
    let values = get_timespan_mean_min_max_values(&connection, &settings, begin, end);
    HttpResponse::Ok().json(values)
}

#[derive(Deserialize)]
struct SensorData {
    id: i32,
    t: f32,
    h: f32,
}

#[get("/api/send")]
async fn send(
    query: web::Query<SensorData>,
) -> HttpResponse {
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
    HttpResponse::Ok().body("OK")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    let timezone: Tz = settings.timezone.parse().unwrap();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    HttpServer::new(move|| {
        let mut tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/*")).unwrap();
        tera.register_function("convert_tz", make_convert_tz(timezone));
        App::new()
            .data(tera)
            .wrap(middleware::Logger::default())
            .service(index)
            .service(simple)
            .service(plots)
            .service(gauges)
            .service(weatherbit)
            .service(table)
            .service(latest)
            .service(history)
            .service(single_mean)
            .service(mean)
            .service(send)
            .service(Files::new("/static", "./static/"))
        })
        .bind("0.0.0.0:5001")?
        .run()
        .await
}
