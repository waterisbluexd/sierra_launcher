use iced::widget::{container, text, column, row, stack};
use iced::{Element, Border, Color, Length, Padding};
use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

const CACHE_FILE: &str = ".cache/sierra/weather.cache";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    temp: String,
    feels_like: String,
    condition: String,
    humidity: String,
    wind_speed: String,
    wind_dir: String,
    hourly: Vec<HourlyData>,
    cached_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyData {
    time: String,
    #[serde(rename = "tempC")]
    temp_c: String,
    #[serde(rename = "windspeedKmph")]
    windspeed_kmph: String,
    #[serde(rename = "precipMM")]
    precip_mm: String,
}

#[derive(Deserialize)]
struct WttrResponse {
    current_condition: Vec<CurrentCondition>,
    weather: Vec<WeatherDay>,
}

#[derive(Deserialize)]
struct CurrentCondition {
    #[serde(rename = "temp_C")]
    temp_c: String,
    #[serde(rename = "FeelsLikeC")]
    feels_like_c: String,
    #[serde(rename = "weatherDesc")]
    weather_desc: Vec<WeatherDesc>,
    humidity: String,
    #[serde(rename = "windspeedKmph")]
    windspeed_kmph: String,
    #[serde(rename = "winddir16Point")]
    winddir16_point: String,
}

#[derive(Deserialize)]
struct WeatherDesc {
    value: String,
}

#[derive(Deserialize)]
struct WeatherDay {
    hourly: Vec<HourlyForecast>,
}

#[derive(Deserialize)]
struct HourlyForecast {
    time: String,
    #[serde(rename = "tempC")]
    temp_c: String,
    #[serde(rename = "windspeedKmph")]
    windspeed_kmph: String,
    #[serde(rename = "precipMM")]
    precip_mm: String,
}

pub struct WeatherPanel {
    weather_data: Arc<Mutex<Option<WeatherData>>>,
    is_updating: Arc<Mutex<bool>>,
}

impl WeatherPanel {
    pub fn new() -> Self {
        let weather_data = Arc::new(Mutex::new(None));
        let is_updating = Arc::new(Mutex::new(true));
        
        let weather_clone = Arc::clone(&weather_data);
        let updating_clone = Arc::clone(&is_updating);

        thread::spawn(move || {
            if let Some(cached) = Self::load_from_cache() {
                *weather_clone.lock().unwrap() = Some(cached.clone());
                if let Ok(age) = SystemTime::now().duration_since(cached.cached_at) {
                    if age.as_secs() < 3600 {
                        *updating_clone.lock().unwrap() = false;
                        return;
                    }
                }
            }

            if let Ok(new_data) = Self::fetch_weather_data() {
                *weather_clone.lock().unwrap() = Some(new_data.clone());
                let _ = Self::save_to_cache(&new_data);
            }
            
            *updating_clone.lock().unwrap() = false;
        });

        Self {
            weather_data,
            is_updating,
        }
    }

    fn get_cache_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CACHE_FILE)
    }

    fn load_from_cache() -> Option<WeatherData> {
        let path = Self::get_cache_path();
        let content = fs::read(&path).ok()?;
        bincode::deserialize(&content).ok()
    }

    fn save_to_cache(data: &WeatherData) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_cache_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let encoded = bincode::serialize(data)?;
        fs::write(&path, encoded)?;
        Ok(())
    }

    fn get_greeting() -> &'static str {
        let hour = Local::now().hour();
        match hour {
            5..=11 => "Good Morning",
            12..=16 => "Good Afternoon",
            17..=20 => "Good Evening",
            _ => "Good Night",
        }
    }

    fn get_weather_art(condition: &str) -> Vec<&'static str> {
        let condition_lower = condition.to_lowercase();
        if condition_lower.contains("sunny") || condition_lower.contains("clear") {
            vec!["    \\   /    ", "     .-.     ", "  — (   ) —  ", "     `-'     ", "    /   \\    "]
        } else if condition_lower.contains("cloudy") {
            vec!["             ", "     .--.    ", "  .-(    ).  ", " (___.__)__) ", "             "]
        } else if condition_lower.contains("rain") || condition_lower.contains("shower") {
            vec!["     .--.    ", "  .-(    ).  ", " (___.__)__) ", "   ‚'‚'‚'‚'   ", "   ‚'‚'‚'‚'   "]
        } else if condition_lower.contains("snow") {
            vec!["     .--.    ", "  .-(    ).  ", " (___.__)__) ", "   * * * * ", "  * * * * "]
        } else if condition_lower.contains("thunder") || condition_lower.contains("storm") {
            vec!["     .--.    ", "  .-(    ).  ", " (___.__)__) ", "   ⚡'‚'⚡'‚'   ", "   ‚'⚡'‚'‚'   "]
        } else {
            vec!["    .--.     ", "   (    )    ", "  (      )   ", " (________)  ", "             "]
        }
    }

    fn fetch_weather_data() -> Result<WeatherData, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(4))
            .build()?;

        let url = "https://wttr.in/?format=j1";
        
        let weather_resp: WttrResponse = client
            .get(url)
            .header("User-Agent", "curl")
            .send()?
            .json()?;

        let current = &weather_resp.current_condition[0];
        let hourly = weather_resp.weather[0].hourly
            .iter()
            .map(|h| HourlyData {
                time: h.time.clone(),
                temp_c: h.temp_c.clone(),
                windspeed_kmph: h.windspeed_kmph.clone(),
                precip_mm: h.precip_mm.clone(),
            })
            .collect();

        Ok(WeatherData {
            temp: current.temp_c.clone(),
            feels_like: current.feels_like_c.clone(),
            condition: current.weather_desc[0].value.clone(),
            humidity: current.humidity.clone(),
            wind_speed: current.windspeed_kmph.clone(),
            wind_dir: current.winddir16_point.clone(),
            hourly,
            cached_at: SystemTime::now(),
        })
    }

    fn format_hourly_forecast(hourly: &[HourlyData]) -> Vec<String> {
        let time_slots = ["0000", "0300", "0600", "0900", "1200", "1500", "1800", "2100"];
        let time_labels = ["12am", "3am", "6am", "9am", "12pm", "3pm", "6pm", "9pm"];
        
        let now = Local::now();
        let current_hour = now.hour();
        let start_slot = (current_hour / 3) % 8;
        
        let mut lines = Vec::new();
        
        let mut header = String::new();
        for i in 0..6 {
            let slot_idx = (start_slot as usize + i) % 8;
            header.push_str(&format!("{:>7}", time_labels[slot_idx]));
        }
        lines.push(header);
        
        let mut temp_line = String::new();
        for i in 0..6 {
            let slot_idx = (start_slot as usize + i) % 8;
            if let Some(hour_data) = hourly.iter().find(|h| h.time == time_slots[slot_idx]) {
                temp_line.push_str(&format!("{:>7}", format!("{}°", hour_data.temp_c)));
            } else {
                temp_line.push_str(&format!("{:>7}", "--"));
            }
        }
        lines.push(temp_line);
        
        let mut wind_line = String::new();
        for i in 0..6 {
            let slot_idx = (start_slot as usize + i) % 8;
            if let Some(hour_data) = hourly.iter().find(|h| h.time == time_slots[slot_idx]) {
                wind_line.push_str(&format!("{:>7}", format!("{}k", hour_data.windspeed_kmph)));
            } else {
                wind_line.push_str(&format!("{:>7}", "--k"));
            }
        }
        lines.push(wind_line);
        
        let mut precip_line = String::new();
        for i in 0..6 {
            let slot_idx = (start_slot as usize + i) % 8;
            if let Some(hour_data) = hourly.iter().find(|h| h.time == time_slots[slot_idx]) {
                precip_line.push_str(&format!("{:>7}", format!("{}mm", hour_data.precip_mm)));
            } else {
                precip_line.push_str(&format!("{:>7}", "--mm"));
            }
        }
        lines.push(precip_line);
        
        lines
    }

    pub fn view<'a, Message: 'a>(
        &self,
        theme: &'a crate::utils::theme::Theme,
        bg_with_alpha: Color,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        let weather_data_guard = self.weather_data.lock().unwrap();
        let is_updating = *self.is_updating.lock().unwrap();

        let content = if let Some(weather_clone) = weather_data_guard.clone() {
            let greeting = Self::get_greeting().to_string();
            let art_lines = Self::get_weather_art(&weather_clone.condition);
            
            let mut art_col = column![].spacing(0);
            for line in art_lines {
                art_col = art_col.push(
                    text(line)
                        .line_height(1.0)
                        .color(Color::from_rgb(0.5, 0.8, 1.0))
                        .font(font)
                        .size(font_size)
                );
            }
            
            let mut info_col = column![].spacing(5).padding(Padding { top: 10.0, right: 0.0, bottom: 0.0, left: 0.0 });
            
            info_col = info_col.push(
                text(greeting)
                    .line_height(1.0)
                    .color(Color::from_rgb(1.0, 0.5, 0.4))
                    .font(font)
                    .size(font_size)
            );
            
            info_col = info_col.push(
                text(weather_clone.condition.clone()) // FIXED: Cloned to take ownership
                    .line_height(1.0)
                    .color(Color::WHITE)
                    .font(font)
                    .size(font_size)
            );
            
            info_col = info_col.push(
                text(format!("{}°C", weather_clone.temp))
                    .line_height(1.0)
                    .color(Color::from_rgb(0.7, 0.9, 1.0))
                    .font(font)
                    .size(font_size * 1.2)
            );
            
            info_col = info_col.push(
                row![
                    text("Wind: ")
                        .line_height(1.0)
                        .color(Color::from_rgb(0.5, 0.5, 0.5))
                        .font(font)
                        .size(font_size),
                    text(format!("{} km/h {}", weather_clone.wind_speed, weather_clone.wind_dir))
                        .line_height(1.0)
                        .color(Color::WHITE)
                        .font(font)
                        .size(font_size),
                ]
                .spacing(5)
            );
            
            info_col = info_col.push(
                row![
                    text("Humidity: ")
                        .line_height(1.0)
                        .color(Color::from_rgb(0.5, 0.5, 0.5))
                        .font(font)
                        .size(font_size),
                    text(format!("{}%", weather_clone.humidity))
                        .line_height(1.0)
                        .color(Color::WHITE)
                        .font(font)
                        .size(font_size),
                ]
                .spacing(5)
            );
            
            if is_updating {
                info_col = info_col.push(
                    text("↻ updating...")
                        .color(Color::from_rgb(0.5, 0.5, 0.5))
                        .font(font)
                        .size(font_size * 0.9)
                );
            }
            
            let top_section = row![
                container(art_col)
                    .width(Length::Fixed(160.0))
                    .padding(Padding { top: 10.0, right: 0.0, bottom: 0.0, left: 45.0 }),
                container(info_col)
                    .width(Length::Fixed(200.0))
                    .padding(Padding { top: 0.0, right:0.0, bottom: 0.0, left: 45.0}),
            ]
            .spacing(1);
            
            let forecast_lines = Self::format_hourly_forecast(&weather_clone.hourly);
            let mut forecast_col = column![].spacing(2).padding(Padding { top: 20.0, right: 30.0, bottom: 0.0, left: 0.0 });
            
            for (i, line) in forecast_lines.into_iter().enumerate() {
                let color = if i == 0 {
                    Color::from_rgb(0.5, 0.8, 1.0)
                } else {
                    Color::WHITE
                };
                
                forecast_col = forecast_col.push(
                    text(line)
                        .line_height(1.0)
                        .color(color)
                        .font(font)
                        .size(font_size * 0.9)
                );
            }
            
            column![
                top_section,
                container(forecast_col)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
            ]
            .spacing(0)
            
        } else {
            column![
                container(
                    text("Loading weather...")
                        .color(Color::from_rgb(0.5, 0.5, 0.5))
                        .font(font)
                        .size(font_size)
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
            ]
        };

        container(
            container(
                stack![
                    container(
                        container(content)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .padding(10)
                            .style(move |_| container::Style {
                                background: None,
                                border: Border {
                                    color: theme.color3,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            })
                    )
                    .padding(iced::padding::top(15))
                    .width(Length::Fill)
                    .height(Length::Fill),
                    
                    container(
                        container(
                            text(" Weather ")
                                .color(theme.color6)
                                .font(font)
                                .size(font_size)
                        )
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .style(move |_| container::Style {
                            background: Some(bg_with_alpha.into()),
                            ..Default::default()
                        })
                    )
                    .padding(iced::padding::left(8).top(5))
                    .width(Length::Shrink)
                    .height(Length::Shrink),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: None,
                ..Default::default()
            })
        )
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .style(move |_| container::Style {
            background: None,
            ..Default::default()
        })
        .into()
    }
}

impl Default for WeatherPanel {
    fn default() -> Self {
        Self::new()
    }
}