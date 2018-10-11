
use chrono::prelude::*;
use chrono_tz::Tz;
use super::schema::sensor_log;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use common::{Settings};
use itertools::sorted;
use time::Duration;


#[derive(Queryable)]
pub struct Log {
    pub sensor_id: i32,
    pub sensor_name: Option<String>,
    pub timestamp: NaiveDateTime,
    pub temperature: Option<f32>,
    pub humidity: Option<f32>,

}

#[derive(Insertable)]
#[table_name="sensor_log"]
pub struct NewLog<'a> {
    pub sensor_id: &'a i32,
    pub sensor_name: &'a str,
    pub timestamp: &'a NaiveDateTime,
    pub temperature: &'a f32,
    pub humidity: &'a f32,
}

pub fn get_latest_values(connection: &MysqlConnection, settings: &Settings) -> Vec<(String, String, String, f32, f32, bool)> {
    use super::schema::sensor_log::dsl::*;
    let mut latest_values: Vec<(String, String, String, f32, f32, bool)> = Vec::new();
    let now = Utc::now().naive_utc();
    let tz: Tz = settings.timezone.parse().unwrap();
    for s_id in sorted(settings.sensor_map.keys()) {
        let s_id: i32 = s_id.parse().expect("Cannot parse s_id");
        let result = sensor_log.filter(sensor_id.eq(s_id))
            .order(timestamp.desc())
            .limit(1)
            .load::<Log>(connection)
            .expect("Error loading sensor logs");
        for log in result {
            //TODO: fix move, multiple usage of log.<attr>
//            println!("{} {} at {}: {}°C {}%", log.sensor_id, &log.sensor_name.unwrap(), &log.timestamp, log.temperature.unwrap(), log.humidity.unwrap());
            let age = now.signed_duration_since(log.timestamp);
            let too_old: bool;
            if age > Duration::seconds(7200) {
                too_old = true;
            } else {
                too_old = false;
            }
            let ts = Utc.from_local_datetime(&log.timestamp).unwrap();
            let ts: String = ts.with_timezone(&tz).to_string();
            latest_values.push((
                log.sensor_id.to_string(),
                log.sensor_name.unwrap(),
                ts,
                log.temperature.unwrap(),
                log.humidity.unwrap(),
                too_old
            ))
        }

    }
    latest_values

}

pub fn get_history(connection: &MysqlConnection, settings: &Settings) -> Vec<(i32, String, Vec<(String, f32, f32)>)> {
    use super::schema::sensor_log::dsl::*;
    let begin = Utc::now().naive_utc() - Duration::days(1);
    let mut history = Vec::new();
    for (s_id, s_name) in sorted(&settings.sensor_map) {
        let s_id: i32 = s_id.parse().expect("Cannot parse s_id");
        let s_name: String = s_name.clone();
        let mut values: Vec<(String, f32, f32)> = Vec::new();
        let result = sensor_log.filter(sensor_id.eq(s_id))
            .filter(timestamp.gt(begin))
            .order_by(timestamp.asc())
            .load::<Log>(connection)
            .expect("Error loading sensor logs");
        for log in result {
            values.push((
                log.timestamp.to_string(),
                log.temperature.unwrap(),
                log.humidity.unwrap()
            ));
        }
        history.push((s_id, s_name, values));
    }
    history
}

pub fn insert_values<'a>(connection: &MysqlConnection, settings: &Settings, sensor_id: &'a i32, temperature: &'a f32, humidity: &'a f32) {
    use schema::sensor_log;
    let new_log = NewLog {
        sensor_id: sensor_id,
        sensor_name: &settings.sensor_map[&sensor_id.to_string()],
        timestamp: &chrono::Utc::now().naive_utc(),
        temperature: temperature,
        humidity: humidity,
    };
    diesel::insert_into(sensor_log::table)
        .values(&new_log)
        .execute(connection)
        .expect("Error saving new values");
}

