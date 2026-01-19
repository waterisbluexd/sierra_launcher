use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use iced::{Font, Color};

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigFile {
    pub font: Option<String>,
    pub font_size: Option<f32>,
    pub use_pywal: Option<bool>,
    pub theme: Option<ThemeConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ThemeConfig {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub border: Option<String>,
    pub accent: Option<String>,
    pub color0: Option<String>,
    pub color1: Option<String>,
    pub color2: Option<String>,
    pub color3: Option<String>,
    pub color4: Option<String>,
    pub color5: Option<String>,
    pub color6: Option<String>,
    pub color7: Option<String>,
    pub color8: Option<String>,
    pub color9: Option<String>,
    pub color10: Option<String>,
    pub color11: Option<String>,
    pub color12: Option<String>,
    pub color13: Option<String>,
    pub color14: Option<String>,
    pub color15: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub use_pywal: bool,
    pub custom_theme: Option<ThemeConfig>,
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
                eprintln!("Using default configuration");
                ConfigFile {
                    font: Some("Monospace".to_string()),
                    font_size: Some(14.0),
                    use_pywal: Some(false),
                    theme: None,
                }
            });

            Config {
                font_name: config_file.font,
                font_size: config_file.font_size,
                use_pywal: config_file.use_pywal.unwrap_or(false),
                custom_theme: config_file.theme,
            }
        } else {
            eprintln!("Config not found at {:?}, using defaults", config_path);
            Self::default()
        }
    }

    pub fn get_font(&self) -> Font {
        self.font_name
            .as_ref()
            .map(|name| {
                let static_name: &'static str = Box::leak(name.clone().into_boxed_str());
                Font::with_name(static_name)
            })
            .unwrap_or(Font::default())
    }

    /// Convert hex color string to iced Color
    pub fn hex_to_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::from_rgb(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                );
            }
        }
        
        // Fallback to white if parsing fails
        Color::WHITE
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_name: Some("Monospace".to_string()),
            font_size: Some(14.0),
            use_pywal: false,
            custom_theme: None,
        }
    }
}