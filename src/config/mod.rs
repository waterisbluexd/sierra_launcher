use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub font: Option<String>,
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
            toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse config: {}", e);
                Self::default()
            })
        } else {
            eprintln!("Config not found at {:?}, using defaults", config_path);
            Self::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: Some("Monospace".to_string()),
            font_size: Some(14.0),
        }
    }
}