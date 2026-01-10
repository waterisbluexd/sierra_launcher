use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub font: Option<String>,
    pub font_size: Option<u16>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Path::new("config/Sierra");
        if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .unwrap_or_else(|_| "".to_string());
            toml::from_str(&content).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: Some("Monospace".to_string()),
            font_size: Some(14),
        }
    }
}
