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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub accent: Color,
    // Add all 16 colors from the palette
    #[allow(dead_code)]
    pub color0: Color,
    #[allow(dead_code)]
    pub color1: Color,
    #[allow(dead_code)]
    pub color2: Color,
    #[allow(dead_code)]
    pub color3: Color,
    #[allow(dead_code)]
    pub color4: Color,
    #[allow(dead_code)]
    pub color5: Color,
    #[allow(dead_code)]
    pub color6: Color,
    #[allow(dead_code)]
    pub color7: Color,
    #[allow(dead_code)]
    pub color8: Color,
    #[allow(dead_code)]
    pub color9: Color,
    #[allow(dead_code)]
    pub color10: Color,
    #[allow(dead_code)]
    pub color11: Color,
    #[allow(dead_code)]
    pub color12: Color,
    #[allow(dead_code)]
    pub color13: Color,
    #[allow(dead_code)]
    pub color14: Color,
    #[allow(dead_code)]
    pub color15: Color,
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
            border: hex_to_color(&self.colors.color7),
            accent: hex_to_color(&self.colors.color4),
            // Convert all palette colors
            color0: hex_to_color(&self.colors.color0),
            color1: hex_to_color(&self.colors.color1),
            color2: hex_to_color(&self.colors.color2),
            color3: hex_to_color(&self.colors.color3),
            color4: hex_to_color(&self.colors.color4),
            color5: hex_to_color(&self.colors.color5),
            color6: hex_to_color(&self.colors.color6),
            color7: hex_to_color(&self.colors.color7),
            color8: hex_to_color(&self.colors.color8),
            color9: hex_to_color(&self.colors.color9),
            color10: hex_to_color(&self.colors.color10),
            color11: hex_to_color(&self.colors.color11),
            color12: hex_to_color(&self.colors.color12),
            color13: hex_to_color(&self.colors.color13),
            color14: hex_to_color(&self.colors.color14),
            color15: hex_to_color(&self.colors.color15),
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