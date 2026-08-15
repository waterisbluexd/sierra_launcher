use serde::Deserialize;
use serde_json;

#[derive(Debug, Clone, Copy, Default)]
pub struct WeatherState {
    pub is_rainy: bool,
    pub is_cloudy: bool,
    pub is_clear: bool,
    pub is_day: bool,
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
}

#[derive(Debug, Deserialize)]
struct MeteoResponse {
    current_weather: MeteoCurrent,
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
        eprintln!("[weather] code={code} is_day={is_day}");

        let (is_rainy, is_cloudy, is_clear) = match code {
            0 => (false, false, true),
            1 | 2 | 3 | 45 | 48 | 71 | 73 | 75 => (false, true, false),
            _ => (true, false, false),
        };

        Some((is_rainy, is_cloudy, is_clear, is_day))
    });

    match result {
        Some((is_rainy, is_cloudy, is_clear, is_day)) => {
            eprintln!("[weather] applying: rainy={is_rainy} cloudy={is_cloudy} clear={is_clear} day={is_day}");
            *state = WeatherState { is_rainy, is_cloudy, is_clear, is_day };
        }
        None => eprintln!("[weather] fetch failed, keeping old state: {state:?}"),
    }
}
