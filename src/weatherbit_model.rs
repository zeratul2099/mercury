extern crate reqwest;
extern crate yaml_rust;




#[derive(Deserialize, Serialize, Debug)]
pub struct WeatherbitCurrent {
    pub data: Vec<CurrentData>,
    pub count: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WeatherbitForecast {
    pub data: Vec<ForecastData>,
    pub city_name: String,
    pub lon: f64,
    pub lat: f64,
    pub timezone: String,
    pub country_code: String,
    pub state_code: String,
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
    //pub last_ob_time: String,
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
    pub precip: f32,
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
pub struct ForecastData {
    pub rh: f32,
    pub pres: f64,
    pub clouds: i64,
    pub clouds_hi: i32,
    pub clouds_mid: i32,
    pub clouds_low: i32,
    pub wind_spd: f64,
    pub wind_gust_spd: f64,
    pub wind_dir: i64,
    pub wind_cdir_full: String,
    pub wind_cdir: String,
    pub slp: f64,
    pub vis: f64,
    pub snow: i64,
    pub uv: f64,
    pub sunrise_ts: i64,
    pub sunset_ts: i64,
    pub moonrise_ts: i64,
    pub moonset_ts: i64,
    pub moon_phase: f32,
    pub moon_phase_lunation: f32,
    pub weather: ForecastDataWeather,
    //pub datetime: DateTime<Utc>,
    pub datetime: String,
    pub valid_date: String,
    pub ts: i64,
    pub temp: f32,
    pub max_temp: f32,
    pub min_temp: f32,
    pub high_temp: Option<f32>,
    pub low_temp: Option<f32>,
    pub app_min_temp: f32,
    pub app_max_temp: f32,
    pub ozone: f32,
    pub snow_depth: i32,
    pub pop: i32,
    pub dewpt: f32,
    pub max_dhi: Option<f32>,
    pub precip: f32,

}
#[derive(Deserialize, Serialize, Debug)]
pub struct CurrentDataWeather {
    pub icon: String,
    pub code: i32,
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ForecastDataWeather {
    pub icon: String,
    pub code: i32,
    pub description: String,
}
