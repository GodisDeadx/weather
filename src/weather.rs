use crate::WeatherInfo;
use dirs::home_dir;
use reqwest;
use serde::Deserialize;
use std::error::Error as StdError;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::exit;

pub fn get_path(filename: &str) -> PathBuf {
    let mut path = home_dir().unwrap();
    #[cfg(target_os = "linux")]
    path.push(".weather/");
    #[cfg(target_os = "windows")]
    path.push(r"AppData\Local\.weather\");
    path.push(filename);
    path
}

#[derive(Debug, Deserialize)]
pub struct Geometry {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug)]
struct LatError(String);

impl std::fmt::Display for LatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LatError {}

struct Values {
    use_celcius: bool,
    use_meters: bool,
}
#[tokio::main]
pub async fn get_weather(
    api_key: &str,
    lat: f64,
    lng: f64,
) -> Result<WeatherInfo, Box<dyn StdError>> {
    let url = format!(
        "https://api.weatherbit.io/v2.0/current?lat={}&lon={}&key={}&include=minutely",
        lat, lng, api_key
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|err| Box::new(err) as Box<dyn StdError>)?;
    if resp.status().is_success() {
        let result: serde_json::Value = resp.json().await.map_err(|err| err.to_string())?;

        let mut temperature = result["data"][0]["temp"].as_f64().unwrap_or(0.0);

        let mut app_temp = result["data"][0]["app_temp"].as_f64().unwrap_or(0.0);

        let weather = result["data"][0]["weather"]["description"]
            .as_str()
            .unwrap_or("unknown");

        let mut wind_speed = result["data"][0]["wind_spd"].as_f64().unwrap_or(0.0);

        let wind_dir = result["data"][0]["wind_cdir_full"]
            .as_str()
            .unwrap_or("unknown");

        let mut dew_point = result["data"][0]["dewpt"].as_f64().unwrap_or(0.0);

        let humidity = result["data"][0]["rh"].as_f64().unwrap_or(0.0);

        let uv_index = result["data"][0]["uv"].as_f64().unwrap_or(0.0);

        let air_quality = result["data"][0]["aqi"].as_f64().unwrap_or(0.0);

        let air_pressure = result["data"][0]["pres"].as_f64().unwrap_or(0.0);

        let city_name = result["data"][0]["city_name"].as_str().unwrap_or("unknown");

        let state_name = result["data"][0]["state_code"]
            .as_str()
            .unwrap_or("unknown");

        let mut precip = result["data"][0]["precip"].as_f64().unwrap_or(0.0);

        let snow = result["data"][0]["snow"].as_f64().unwrap_or(0.0);
        precip = precip + snow;

        let sunrise = result["data"][0]["sunrise"].as_str().unwrap_or("unknown");

        let sunset = result["data"][0]["sunset"].as_str().unwrap_or("unknown");

        let air_pressure = air_pressure * 0.02953; // Convert mb to hPa

        let weather_info = WeatherInfo {
            location: format!("{}, {}", city_name, state_name),
            sunrise: sunrise.to_string(),
            sunset: sunset.to_string(),
            temperature: temperature,
            app_temp: app_temp,
            dew_point: dew_point,
            weather: weather.to_string(),
            precip: precip,
            wind_speed: wind_speed,
            wind_direction: wind_dir.to_string(),
            air_pressure: air_pressure,
            humidity: humidity,
            uv_index: uv_index,
            air_quality: air_quality,
        };
        return Ok(weather_info);
    }

    eprintln!("Error");
    exit(1);
}

#[tokio::main]
pub async fn get_coords(
    api_key: &str,
    city: &str,
    state: &str,
) -> Result<Geometry, Box<dyn StdError>> {
    let url = format!(
        "https://api.opencagedata.com/geocode/v1/json?q={},{}&key={}",
        city, state, api_key
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|err| Box::new(err) as Box<dyn StdError>)?;

    if resp.status().is_success() {
        let result: serde_json::Value = resp.json().await.map_err(|err| err.to_string())?;
        let lat = result["results"][0]["geometry"]["lat"]
            .as_f64()
            .unwrap_or(0.0);
        let lng = result["results"][0]["geometry"]["lng"]
            .as_f64()
            .unwrap_or(0.0);
        let geometry = Geometry { lat, lng };
        return Ok(geometry);
    }

    return Err(Box::new(LatError("Error".to_string())) as Box<dyn std::error::Error>);
}

fn create_dir() {
    let path = get_path("");
    std::fs::create_dir_all(path).expect("Failed to create directory");
}
