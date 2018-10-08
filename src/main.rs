#![allow(proc_macro_derive_resolution_fallback,dead_code)]
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;
extern crate itertools;
#[macro_use] extern crate lazy_static;

#[macro_use]
extern crate diesel;
extern crate chrono;

pub mod common;
pub mod schema;
pub mod models;

use self::models::*;
use rocket_contrib::Json;
use common::{get_settings,check_notification,establish_connection};



#[derive(FromForm)]
struct SendQuery {
    id: i32,
    t: f32,
    h: f32,
}

fn main() {
    rocket::ignite().mount("/", routes![send,latest]).launch();
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
fn latest() -> Json<Vec<(String, String, String, f32, f32)>> {
    let settings = get_settings();
    let connection = establish_connection(&settings);
    let values = get_latest_values(&connection, &settings);
    Json(values)
}

