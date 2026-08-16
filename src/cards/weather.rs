use cacache;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::SystemTime;

const WEATHER_CACHE_KEY: &str = "weather-current";
const WEATHER_CACHE_TTL_MS: u128 = 30 * 60 * 1000;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeatherState {
    pub is_rainy: bool,
    pub is_cloudy: bool,
    pub is_clear: bool,
    pub is_day: bool,
    pub condition: String,
    pub temperature: f64,
}

#[derive(Debug, Deserialize)]
struct IpLocation {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize)]
struct FreeIpLocation {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize)]
struct MeteoCurrent {
    weathercode: i32,
    is_day: i32,
    temperature: f64,
}

#[derive(Debug, Deserialize)]
struct MeteoResponse {
    current_weather: MeteoCurrent,
}

fn condition_label(code: i32) -> &'static str {
    match code {
        0 => "Clear",
        1 => "Mostly Clear",
        2 => "Partly Cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing Drizzle",
        61 => "Light Rain",
        63 => "Rain",
        65 => "Heavy Rain",
        66 | 67 => "Freezing Rain",
        71 | 73 | 75 => "Snow",
        77 => "Snow Grains",
        80 | 81 | 82 => "Rain Showers",
        85 | 86 => "Snow Showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm + Hail",
        _ => "UNKNOWN ERROR",
    }
}

async fn try_ipwho(client: &reqwest::Client) -> Option<IpLocation> {
    let r = client.get("https://ipwho.is/").send().await.ok()?;
    if !r.status().is_success() {
        return None;
    }
    let b = r.bytes().await.ok()?;
    serde_json::from_slice::<IpLocation>(&b).ok()
}

async fn try_freeipapi(client: &reqwest::Client) -> Option<FreeIpLocation> {
    let r = client.get("https://freeipapi.com/api/json/").send().await.ok()?;
    if !r.status().is_success() {
        return None;
    }
    let b = r.bytes().await.ok()?;
    serde_json::from_slice::<FreeIpLocation>(&b).ok()
}

pub fn update_weather(state: &mut WeatherState) {
    eprintln!("[weather] starting update");

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
        Ok(rt) => rt,
        Err(e) => { eprintln!("[weather] runtime build failed: {e}"); return; }
    };

    let result = rt.block_on(async {
        let client = match reqwest::Client::builder()
            .user_agent("sierra-launcher/0.1")
            .timeout(std::time::Duration::from_secs(10))
            .build() {
            Ok(c) => c,
            Err(e) => { eprintln!("[weather] client build failed: {e}"); return None; }
        };

        eprintln!("[weather] fetching ip location...");
        let loc = match client
            .get("https://ipwho.is/")
            .send()
            .await
        {
            Ok(r) => {
                eprintln!("[weather] ip location status: {}", r.status());
                if !r.status().is_success() {
                    eprintln!("[weather] ip location bad status");
                    return None;
                }
                match r.bytes().await {
                    Ok(b) => {
                        eprintln!("[weather] ip location body: {}", String::from_utf8_lossy(&b));
                        match serde_json::from_slice::<IpLocation>(&b) {
                            Ok(l) => l,
                            Err(e) => { eprintln!("[weather] ip location parse failed: {e}"); return None; }
                        }
                    }
                    Err(e) => { eprintln!("[weather] ip location bytes failed: {e}"); return None; }
                }
            }
            Err(e) => { eprintln!("[weather] ip location request failed: {e}"); return None; }
        };

        eprintln!("[weather] got lat={} lon={}", loc.latitude, loc.longitude);

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
            loc.latitude, loc.longitude
        );

        eprintln!("[weather] fetching meteo: {url}");
        let meteo = match client.get(&url).send().await {
            Ok(r) => {
                eprintln!("[weather] meteo status: {}", r.status());
                match r.bytes().await {
                    Ok(b) => {
                        eprintln!("[weather] meteo body: {}", String::from_utf8_lossy(&b));
                        match serde_json::from_slice::<MeteoResponse>(&b) {
                            Ok(m) => m,
                            Err(e) => { eprintln!("[weather] meteo parse failed: {e}"); return None; }
                        }
                    }
                    Err(e) => { eprintln!("[weather] meteo bytes failed: {e}"); return None; }
                }
            }
            Err(e) => { eprintln!("[weather] meteo request failed: {e}"); return None; }
        };

        let code = meteo.current_weather.weathercode;
        let is_day = meteo.current_weather.is_day == 1;
        let temperature = meteo.current_weather.temperature;
        let condition = condition_label(code).to_string();
        eprintln!("[weather] code={code} is_day={is_day} temp={temperature} condition={condition}");

        let (is_rainy, is_cloudy, is_clear) = match code {
            0 => (false, false, true),
            1 | 2 | 3 | 45 | 48 | 71 | 73 | 75 => (false, true, false),
            _ => (true, false, false),
        };

        Some((is_rainy, is_cloudy, is_clear, is_day, condition, temperature))
    });

    match result {
        Some((is_rainy, is_cloudy, is_clear, is_day, condition, temperature)) => {
            eprintln!("[weather] applying: rainy={is_rainy} cloudy={is_cloudy} clear={is_clear} day={is_day} temp={temperature} condition={condition}");
            *state = WeatherState {
                is_rainy,
                is_cloudy,
                is_clear,
                is_day,
                condition,
                temperature,
            };

            ensure_cache_dir();
            let dir = cache_dir();
            let json = match serde_json::to_vec(state) {
                Ok(j) => j,
                Err(e) => { eprintln!("[weather] cache serialize failed: {e}"); return; }
            };
            if let Err(e) = cacache::write_sync(&dir, WEATHER_CACHE_KEY, &json) {
                eprintln!("[weather] cache write failed: {e}");
            }
        }
        None => eprintln!("[weather] fetch failed, keeping old state: {state:?}"),
    }
}

fn cache_dir() -> String {
    std::env::var("HOME")
        .map(|home| format!("{}/.local/share/sierra-launcher/cache", home))
        .unwrap_or_else(|_| "/tmp/sierra-launcher-cache".to_string())
}

fn ensure_cache_dir() {
    let dir = cache_dir();
    let _ = std::fs::create_dir_all(&dir);
}

pub fn load_weather_from_cache() -> Option<WeatherState> {
    let dir = cache_dir();
    let bytes = cacache::read_sync(&dir, WEATHER_CACHE_KEY).ok()?;

    let meta = cacache::metadata_sync(&dir, WEATHER_CACHE_KEY).ok()??;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_millis();

    if now.saturating_sub(meta.time) > WEATHER_CACHE_TTL_MS {
        eprintln!("[weather] cache stale, ignoring");
        return None;
    }

    serde_json::from_slice(&bytes).ok()
}
