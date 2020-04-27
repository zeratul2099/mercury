extern crate reqwest;
extern crate yaml_rust;

use chrono::{DateTime, Utc};


#[derive(Deserialize, Serialize, Debug)]
pub struct WeatherbitCurrent {
    pub data: Vec<CurrentData>,
    pub count: i64,
}


#[derive(Deserialize, Serialize, Debug)]
pub struct CurrentData {
    pub rh: f32,
    pub pod: String,
    pub lon: f64,
    pub pres: f64,
    pub timezone: String,
    //pub ob_time: DateTime<Utc>,
    pub ob_time: String,
    pub country_code: String,
    pub clouds: i64,
    pub ts: i64,
    pub solar_rad: f64,
    pub state_code: String,
    pub city_name: String,
    pub wind_spd: f64,
    //pub last_ob_time: DateTime<Utc>,
    pub last_ob_time: String,
    pub wind_cdir_full: String,
    pub wind_cdir: String,
    pub slp: f64,
    pub vis: f64,
    pub h_angle: f64,
    pub sunset: String,
    pub dni: f64,
    pub dewpt: f64,
    pub snow: i64,
    pub uv: f64,
    pub precip: i64,
    pub wind_dir: i64,
    pub sunrise: String,
    pub ghi: f64,
    pub dhi: f64,
    pub aqi: i64,
    pub lat: f64,
    pub weather: CurrentDataWeather,
    //pub datetime: DateTime<Utc>,
    pub datetime: String,
    pub temp: f32,
    pub station: String,
    pub elev_angle: f64,
    pub app_temp: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CurrentDataWeather {
    pub icon: String,
    pub code: String,
    pub description: String,
}
