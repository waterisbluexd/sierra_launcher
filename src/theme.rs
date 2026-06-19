use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Special {
    pub background: String,
    pub foreground: String,
    #[serde(default)]
    pub cursor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Colors {
    pub color0: String,
    pub color1: String,
    pub color2: String,
    pub color3: String,
    pub color4: String,
    pub color5: String,
    pub color6: String,
    pub color7: String,
    pub color8: String,
    pub color9: String,
    pub color10: String,
    pub color11: String,
    pub color12: String,
    pub color13: String,
    pub color14: String,
    pub color15: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub special: Special,
    pub colors: Colors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            special: Special {
                background: "#1e1e2e".to_string(),
                foreground: "#cdd6f4".to_string(),
                cursor: "#cdd6f4".to_string(),
            },
            colors: Colors {
                color0: "#45475a".to_string(),
                color1: "#f38ba8".to_string(),
                color2: "#a6e3a1".to_string(),
                color3: "#f9e2af".to_string(),
                color4: "#89b4fa".to_string(),
                color5: "#f5c2e7".to_string(),
                color6: "#94e2d5".to_string(),
                color7: "#bac2de".to_string(),
                color8: "#585b70".to_string(),
                color9: "#f38ba8".to_string(),
                color10: "#a6e3a1".to_string(),
                color11: "#f9e2af".to_string(),
                color12: "#89b4fa".to_string(),
                color13: "#f5c2e7".to_string(),
                color14: "#94e2d5".to_string(),
                color15: "#a6adc8".to_string(),
            },
        }
    }
}

impl Theme {
    pub fn path() -> PathBuf {
        PathBuf::from(env::var("HOME").expect("HOME environment variable not set"))
            .join(".cache")
            .join("wal")
            .join("colors.json")
    }

    pub fn load() -> Self {
        let path = Self::path();

        match fs::read_to_string(&path) {
            Ok(contents) if !contents.trim().is_empty() => {
                match serde_json::from_str::<Theme>(&contents) {
                    Ok(theme) => theme,
                    Err(err) => {
                        eprintln!("Failed to parse pywal colors.json: {err}");
                        Self::default()
                    }
                }
            }
            Ok(_) => {
                println!("colors.json is empty, using default theme");
                Self::default()
            }
            Err(_) => {
                println!("No pywal colors.json found, using default theme");
                Self::default()
            }
        }
    }
}
