use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use iced::Font;

#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub font: Option<String>,
    pub font_size: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub font_name: Option<String>,  // Store the font NAME, not Font object
    pub font_size: Option<f32>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config/sierra/Sierra")
        } else {
            PathBuf::from("config/Sierra")
        };

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .unwrap_or_else(|_| "".to_string());
            
            let config_file: ConfigFile = toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse config: {}", e);
                ConfigFile {
                    font: Some("Monospace".to_string()),
                    font_size: Some(14.0),
                }
            });

            Config {
                font_name: config_file.font,
                font_size: config_file.font_size,
            }
        } else {
            eprintln!("Config not found at {:?}, using defaults", config_path);
            Self::default()
        }
    }

    // Helper method to get Font from the stored name
    pub fn get_font(&self) -> Font {
        self.font_name
            .as_ref()
            .map(|name| {
                // Leak the string to get 'static lifetime
                // This is acceptable since fonts are loaded once and rarely change
                let static_name: &'static str = Box::leak(name.clone().into_boxed_str());
                Font::with_name(static_name)
            })
            .unwrap_or(Font::default())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_name: Some("Monospace".to_string()),
            font_size: Some(14.0),
        }
    }
}