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

#[path = "../common.rs"]
mod common;
#[path = "../models.rs"]
pub mod models;
#[path = "../schema.rs"]
pub mod schema;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware, error};
use tera::Tera;
use crate::common::{establish_connection, get_settings};
use crate::models::get_latest_values;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/*")).unwrap();
        App::new()
            .data(tera)
            .wrap(middleware::Logger::default())
            .service(index)
            .service(simple)
        })
        .bind("0.0.0.0:6502")?
        .run()
        .await
}
#[get("/simple")]
async fn simple(
    tmpl: web::Data<tera::Tera>,
) -> HttpResponse {
    let mut context = tera::Context::new();
    let settings = get_settings();
    let connection = establish_connection(&settings);
    values = get_latest_values(&connection, &settings, 1, None) 
    context.insert("latest_values", &values);
    let s = tmpl.render("simple", &context)
        .map_err(|_| error::ErrorInternalServerError("TemplateError"))?
    HttpResponse::Ok().content_type("text/html").body(s)
