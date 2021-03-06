
extern crate reqwest;
extern crate yaml_rust;

use self::yaml_rust::yaml;
use chrono_tz::Tz;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Mutex;

lazy_static! {
    static ref NOTIFIED: Mutex<HashSet<usize>> = Mutex::new(HashSet::new());
}

pub struct Settings {
    #[allow(dead_code)]
    pub database: String,
    pub sensor_map: HashMap<String, String>,
    pub timezone: String,
    pub weatherbit_api_key: String,
    pub lat_lon: (f64, f64),
    pub pa_app_token: String,
    pub pa_user_key: String,
    pub notification_constraints: Vec<(i64, String, f64, String)>,
}

pub fn get_settings() -> Settings {
    let mut file = File::open("settings.yaml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let docs = yaml::YamlLoader::load_from_str(&contents).unwrap();
    //value.push_str(docs[0][key].as_str().unwrap());
    let lat: f64 = docs[0]["lat_lon"][0].as_f64().unwrap();
    let lon: f64 = docs[0]["lat_lon"][1].as_f64().unwrap();
    let mut sensor_map = HashMap::new();
    for (k, v) in docs[0]["sensor_map"].as_hash().unwrap() {
        sensor_map.insert(
            String::from(k.as_str().unwrap()),
            String::from(v.as_str().unwrap()),
        );
    }
    let mut constr: Vec<(i64, String, f64, String)> = Vec::new();
    for vector in docs[0]["notification_constraints"].as_vec().unwrap() {
        constr.push((
            vector[0].as_i64().unwrap(),
            String::from(vector[1].as_str().unwrap()),
            vector[2].as_f64().unwrap(),
            String::from(vector[3].as_str().unwrap()),
        ));
    }
    let settings = Settings {
        database: String::from(str::replace(
            docs[0]["database"].as_str().unwrap(),
            "+pymysql",
            "",
        )),
        timezone: String::from(docs[0]["timezone"].as_str().unwrap()),
        weatherbit_api_key: String::from(docs[0]["weatherbit_api_key"].as_str().unwrap()),
        lat_lon: (lat, lon),
        pa_app_token: String::from(docs[0]["pa_app_token"].as_str().unwrap()),
        pa_user_key: String::from(docs[0]["pa_user_key"].as_str().unwrap()),
        sensor_map: sensor_map,
        notification_constraints: constr,
    };
    settings
}

pub fn establish_connection(settings: &Settings) -> MysqlConnection {
    MysqlConnection::establish(&settings.database)
        .expect(&format!("Error connection to {}", settings.database))
}

fn send_pushover_message(settings: &Settings, message: String) {
    let postdata: String = format!(
        "token={}&user={}&message={}",
        &settings.pa_app_token, &settings.pa_user_key, message
    );
    let client = reqwest::Client::new();
    let _res = client
        .post("https://api.pushover.net/1/messages.json")
        .body(postdata)
        .send();
}

pub fn check_notification(settings: &Settings, sensor: i64, vtype: &String, value: f64) {
    let ts = chrono::Utc::now();
    let tz: Tz = settings.timezone.parse().unwrap();
    let ts = ts
        .with_timezone(&tz)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    for (idx, (csensor, ctype, cvalue, cmp)) in settings.notification_constraints.iter().enumerate()
    {
        if sensor == *csensor && ctype == vtype {
            let sensor_name: &String = &settings.sensor_map[&sensor.to_string()];
            let cmp_word: String;
            let msg: String;
            if !NOTIFIED.lock().unwrap().contains(&idx) {
                if (cmp.eq("+") && value > *cvalue) || (cmp.eq("-") && value < *cvalue) {
                    // notify
                    if cmp.eq("+") {
                        cmp_word = "over".to_string();
                    } else {
                        cmp_word = "below".to_string();
                    };
                    msg = format!(
                        "{}: {} is {} limit of {} ({})",
                        sensor_name, vtype, cmp_word, cvalue, ts
                    );
                    println!("{}", msg);
                    send_pushover_message(&settings, msg);
                    println!("{}: {}, {}, {}, {}", idx, csensor, ctype, cvalue, cmp);
                    NOTIFIED.lock().unwrap().insert(idx);
                }
            } else if (cmp.eq("+") && value < cvalue - 0.5) || (cmp == "-" && value > cvalue + 0.5)
            {
                if cmp.eq("+") {
                    cmp_word = "below".to_string();
                } else {
                    cmp_word = "over".to_string();
                };
                msg = format!(
                    "{} all clear: {} is {} limit of {} again ({})",
                    sensor_name, vtype, cmp_word, cvalue, ts
                );
                println!("{}", msg);
                send_pushover_message(&settings, msg);
                NOTIFIED.lock().unwrap().remove(&idx);
            }
        }
    }
}
