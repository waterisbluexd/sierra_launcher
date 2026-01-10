use iced::Color;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct WalColors {
    pub special: SpecialColors,
    pub colors: PaletteColors,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpecialColors {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaletteColors {
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

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub accent: Color,
}

impl WalColors {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")?;
        let path = PathBuf::from(home).join(".cache/wal/colors.json");
        let contents = fs::read_to_string(path)?;
        let colors: WalColors = serde_json::from_str(&contents)?;
        Ok(colors)
    }

    pub fn to_theme(&self) -> Theme {
        Theme {
            background: hex_to_color(&self.special.background),
            foreground: hex_to_color(&self.special.foreground),
            border: hex_to_color(&self.colors.color8),
            accent: hex_to_color(&self.colors.color4),
        }
    }
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}