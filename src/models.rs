use super::schema::sensor_log;
use chrono::prelude::*;
use chrono_tz::Tz;
use common::Settings;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
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
#[table_name = "sensor_log"]
pub struct NewLog<'a> {
    pub sensor_id: &'a i32,
    pub sensor_name: &'a str,
    pub timestamp: &'a NaiveDateTime,
    pub temperature: &'a f32,
    pub humidity: &'a f32,
}

pub fn get_latest_values(
    connection: &MysqlConnection,
    settings: &Settings,
    limit: i64,
    filter_sensor_id: Option<i32>,
) -> Vec<(String, String, String, f32, f32, bool)> {
    use super::schema::sensor_log::dsl::*;
    let mut latest_values: Vec<(String, String, String, f32, f32, bool)> = Vec::new();
    let now = Utc::now().naive_utc();
    let tz: Tz = settings.timezone.parse().unwrap();
    for s_id in sorted(settings.sensor_map.keys()) {
        let s_id: i32 = s_id.parse().expect("Cannot parse s_id");
        match filter_sensor_id {
            None => (),
            Some(fs_id) => if fs_id != s_id {
                continue;
            }
        };
        let result = sensor_log
            .filter(
                sensor_id.eq(s_id)
                .and(temperature.is_not_null())
                )
            .order(timestamp.desc())
            .limit(limit)
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
                log.temperature.expect("Invalid value for temperature"),
                log.humidity.expect("Unvalid value for humidity"),
                too_old,
            ))
        }
    }
    latest_values
}

pub fn get_history(
    connection: &MysqlConnection,
    settings: &Settings,
    days: i64,
) -> Vec<(i32, String, Vec<(String, f32, f32)>)> {
    use super::schema::sensor_log::dsl::*;
    let begin = Utc::now().naive_utc() - Duration::days(days);
    let tz: Tz = settings.timezone.parse().unwrap();
    let mut history = Vec::new();
    for (s_id, s_name) in sorted(&settings.sensor_map) {
        let s_id: i32 = s_id.parse().expect("Cannot parse s_id");
        let s_name: String = s_name.clone();
        let mut values: Vec<(String, f32, f32)> = Vec::new();
        let result = sensor_log
            .filter(sensor_id.eq(s_id))
            .filter(timestamp.gt(begin))
            .order_by(timestamp.asc())
            .load::<Log>(connection)
            .expect("Error loading sensor logs");
        for log in result {
            let ts = Utc.from_local_datetime(&log.timestamp).unwrap();
            let t = match log.temperature {
                None => continue,
                Some(t) => t,
            };
            let h = match log.humidity {
                None => continue,
                Some(h) => h,
            };
            let ts: String = ts.with_timezone(&tz).naive_local().to_string();
            values.push((ts, t, h));
        }
        history.push((s_id, s_name, values));
    }
    history
}

pub fn insert_values<'a>(
    connection: &MysqlConnection,
    settings: &Settings,
    sensor_id: &'a i32,
    temperature: &'a f32,
    humidity: &'a f32,
) {
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

pub fn get_day_mean_values(
    connection: &MysqlConnection,
    s_id: &i32,
//    day: NaiveDate,
) -> (f32, f32) {
    use super::schema::sensor_log::dsl::*;
    //let end = day - Duration::days(1);
    //let begin = day;
    let begin = Utc::now().naive_utc() - Duration::days(1);
    let end = Utc::now().naive_utc();
    let result = sensor_log
        .filter(sensor_id.eq(s_id))
        .filter(timestamp.gt(begin))
        .filter(timestamp.lt(end))
        .order_by(timestamp.asc())
        .load::<Log>(connection)
        .expect("Error loading sensor logs");
    let mut t_sum: f32 = 0.0;
    let mut h_sum: f32 = 0.0;
    let mut t_len: f32 = 0.0;
    let mut h_len: f32 = 0.0;

    for log in result {
        t_sum += log.temperature.unwrap();
        h_sum += log.humidity.unwrap();
        t_len += 1.0;
        h_len += 1.0;
    }
    let t_mean: f32 = t_sum / t_len;
    let h_mean: f32 = h_sum / h_len;
    (t_mean, h_mean)
}
