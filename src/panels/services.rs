use iced::widget::{container, text, stack, row, column, vertical_slider, slider, button};
use iced::{Element, Border, Color, Length, Background, Shadow};
use crate::utils::theme::Theme;
use crate::Message;
use std::process::Command;
use regex::Regex;

pub struct ServicesPanel {
    pub volume_value: f32,
    pub brightness_value: f32,
    pub slider_height: f32,
    previous_volume_value: f32,
    is_muted: bool,
    previous_brightness_value: f32,
    is_min_brightness: bool,
    pub wifi_enabled: bool,
    pub wifi_name: String,
}

impl ServicesPanel {
    pub fn new() -> Self {
        let volume_value = Self::get_volume().unwrap_or(50.0);
        let brightness_value = Self::get_brightness().unwrap_or(50.0);
        let (wifi_enabled, wifi_name) = Self::get_wifi_status();

        Self {
            volume_value,
            brightness_value,
            slider_height: 120.0,
            previous_volume_value: volume_value,
            is_muted: false,
            previous_brightness_value: brightness_value,
            is_min_brightness: false,
            wifi_enabled,
            wifi_name,
        }
    }

    fn get_volume() -> Option<f32> {
        let output = Command::new("pactl").arg("get-sink-volume").arg("@DEFAULT_SINK@").output().ok()?;
        let output_str = String::from_utf8(output.stdout).ok()?;
        let re = Regex::new(r"(\d+)%").unwrap();
        let caps = re.captures(&output_str)?;
        let value_str = caps.get(1)?.as_str();
        value_str.parse::<f32>().ok()
    }

    fn get_brightness() -> Option<f32> {
        let current_output = Command::new("brightnessctl").arg("g").output().ok()?;
        let current_str = String::from_utf8(current_output.stdout).ok()?.trim().to_string();
        let current = current_str.parse::<f32>().ok()?;

        let max_output = Command::new("brightnessctl").arg("m").output().ok()?;
        let max_str = String::from_utf8(max_output.stdout).ok()?.trim().to_string();
        let max = max_str.parse::<f32>().ok()?;

        if max > 0.0 {
            Some((current / max) * 100.0)
        } else {
            None
        }
    }

    fn get_wifi_status() -> (bool, String) {
        // Try nmcli first (NetworkManager)
        if let Ok(output) = Command::new("nmcli")
            .args(&["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
            .output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines() {
                    if line.starts_with("yes:") {
                        let ssid = line.strip_prefix("yes:").unwrap_or("Connected");
                        return (true, ssid.to_string());
                    }
                }
            }
        }

        // Fallback to iwgetid
        if let Ok(output) = Command::new("iwgetid").arg("-r").output() {
            if let Ok(ssid) = String::from_utf8(output.stdout) {
                let ssid = ssid.trim();
                if !ssid.is_empty() {
                    return (true, ssid.to_string());
                }
            }
        }

        // Check if WiFi is disabled
        if let Ok(output) = Command::new("nmcli").args(&["radio", "wifi"]).output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if stdout.trim() == "disabled" {
                    return (false, "WiFi Off".to_string());
                }
            }
        }

        // WiFi is on but not connected
        (true, "No Network".to_string())
    }

    pub fn toggle_wifi(&mut self) {
        self.wifi_enabled = !self.wifi_enabled;
        
        if self.wifi_enabled {
            // Enable WiFi
            let _ = Command::new("nmcli").args(&["radio", "wifi", "on"]).output();
            std::thread::sleep(std::time::Duration::from_millis(500));
            self.refresh_wifi_status();
        } else {
            // Disable WiFi
            let _ = Command::new("nmcli").args(&["radio", "wifi", "off"]).output();
            self.wifi_name = "WiFi Off".to_string();
        }
    }

    pub fn refresh_wifi_status(&mut self) {
        let (enabled, name) = Self::get_wifi_status();
        self.wifi_enabled = enabled;
        self.wifi_name = name;
    }

    pub fn view<'a>(
        &'a self,
        theme: &'a Theme,
        bg_with_alpha: Color,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        
        // --- 1. DETERMINE WIFI STYLING COLORS ---
        let is_connected = self.wifi_enabled && self.wifi_name != "No Network" && self.wifi_name != "WiFi Off";
        
        let active_accent = if is_connected { theme.color2 } else { theme.color3 };
        let inactive_accent = theme.color8; // Grey for disabled

        // Dynamic State: Filled vs Outline
        let (wifi_text_color, wifi_bg_color, wifi_border_color) = if self.wifi_enabled {
            // ACTIVE: Filled background, Dark Text
            (theme.color0, active_accent, Color::TRANSPARENT)
        } else {
            // INACTIVE: Transparent background, Grey Text, Grey Border
            (inactive_accent, Color::TRANSPARENT, inactive_accent)
        };

        let wifi_icon_str = if self.wifi_enabled { "󰤨" } else { "󰤮" };

        // --- 2. BUILD THE WIFI BUTTON CONTENT ---
        let wifi_button_content = container(
            row![
                // Icon
                container(
                    text(wifi_icon_str)
                        .color(wifi_text_color)
                        .font(font)
                        .size(font_size * 2.2)
                        .center()
                )
                .padding(iced::padding::right(12))
                .align_y(iced::alignment::Vertical::Center),

                // Text Info
                column![
                    text("CONNECTION")
                        .color(wifi_text_color)
                        .size(font_size * 0.65)
                        .font(font),
                    text(if self.wifi_name.len() > 14 { 
                        format!("{}..", &self.wifi_name[..12]) 
                    } else { 
                        self.wifi_name.clone() 
                    })
                        .color(wifi_text_color)
                        .size(font_size * 0.9)
                        .font(font),
                ]
                .spacing(2)
                .align_x(iced::alignment::Horizontal::Left)
                // Removed .justify_y() as it does not exist in Column
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::padding::left(15).right(5))
        .align_y(iced::alignment::Vertical::Center);

        // --- 3. ASSEMBLE LEFT PANEL ---
        let left_part = container(
            column![
                // Top Row: WiFi + Airplane/Mute
                container(
                    row![
                        // WiFi Button Wrapper
                        container(
                            button(wifi_button_content)
                                .on_press(Message::WifiToggle)
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .style(move |_theme, status| {
                                    match status {
                                        iced::widget::button::Status::Hovered => button::Style {
                                            // On hover: If active, slight transparency. If inactive, slight color fill.
                                            background: Some(if self.wifi_enabled {
                                                let mut c = wifi_bg_color; c.a = 0.9; c.into()
                                            } else {
                                                let mut c = active_accent; c.a = 0.1; c.into()
                                            }),
                                            border: Border {
                                                color: active_accent,
                                                width: 2.0,
                                                radius: 0.0.into(),
                                            },
                                            text_color: wifi_text_color,
                                            ..Default::default()
                                        },
                                        iced::widget::button::Status::Pressed => button::Style {
                                            background: Some(active_accent.into()),
                                            border: Border { color: active_accent, width: 2.0, radius: 0.0.into() },
                                            text_color: theme.color0,
                                            ..Default::default()
                                        },
                                        _ => button::Style {
                                            background: Some(wifi_bg_color.into()),
                                            border: Border {
                                                color: wifi_border_color,
                                                width: 1.5,
                                                radius: 0.0.into(),
                                            },
                                            text_color: wifi_text_color,
                                            ..Default::default()
                                        }
                                    }
                                }),
                        )
                        .padding(iced::padding::right(10)) 
                        .width(Length::Fill)
                        .height(Length::Fill),

                        // Airplane / Mute Button
                        container(
                            button(
                                container(
                                    text("󰀝")
                                        .color(theme.color1) 
                                        .font(font)
                                        .size(font_size * 2.0)
                                        .center()
                                )
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x(Length::Fill) 
                                .center_y(Length::Fill) 
                            )
                            .on_press(Message::AudioMuteToggle)
                            .style(move |_, status| {
                                let border_c = theme.color1;
                                match status {
                                    iced::widget::button::Status::Hovered => button::Style {
                                        background: Some(Color::from_rgba(border_c.r, border_c.g, border_c.b, 0.1).into()),
                                        border: Border {
                                            color: border_c,
                                            width: 2.0,
                                            radius: 0.0.into(),
                                        },
                                        ..Default::default()
                                    },
                                    _ => button::Style {
                                        background: Some(Color::TRANSPARENT.into()),
                                        border: Border {
                                            color: theme.color2, 
                                            width: 1.5,
                                            radius: 0.0.into(),
                                        },
                                        ..Default::default()
                                    }
                                }
                            }),
                        )
                        .width(Length::Fixed(85.0))
                        .height(Length::Fill)
                    ]
                )
                .padding(10)
                .width(Length::Fill)
                .height(Length::FillPortion(1)), 

                container(
                    row![
                        container(text("y").color(theme.color3))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            // FIX: Changed |_,_| to |_| as container style takes 1 arg
                            .style(move |_| container::Style {
                                border: Border { color: theme.color3, width: 2.0, radius: 0.0.into() },
                                ..Default::default()
                            }),
                        container(text("y.2").color(theme.color3))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            // FIX: Changed |_,_| to |_| as container style takes 1 arg
                            .style(move |_| container::Style {
                                border: Border { color: theme.color3, width: 2.0, radius: 0.0.into() },
                                ..Default::default()
                            }),
                    ]
                    .spacing(10)
                )
                .width(Length::Fill)
                .height(Length::FillPortion(1)), 
            ]
            .spacing(10)
        )
        .padding(iced::padding::top(10).bottom(3).right(12).left(10))
        .width(Length::Fill)
        .height(Length::Fill);

        // --- RIGHT PANEL (Sliders) ---
        let volume_icon = if self.is_muted {""} else if self.volume_value <= 33.0 {""} else if self.volume_value <= 66.0 {"" } else {""};
        let brightness_icon = if self.brightness_value <= 33.0 { "󰃞" } else if self.brightness_value <= 66.0 { "󰃟" } else { "󰃠" };

        let volume_column = column![
            container(
                text(format!("{}%", self.volume_value as i32))
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(iced::padding::top(6).bottom(4)),
            
            vertical_slider(0.0..=100.0, self.volume_value, Message::VolumeChanged)
                .height(Length::Fixed(self.slider_height))
                .width(20.0)
                .step(1.0)
                .style(move |_theme, _status| slider::Style {
                    rail: slider::Rail {
                        backgrounds: (
                            iced::Background::Color(theme.color4),
                            iced::Background::Color(Color::from_rgba(theme.color6.r, theme.color6.g, theme.color6.b, 0.3)),
                        ),
                        width: 20.0,
                        border: Border { radius: 0.0.into(), ..Default::default() },
                    },
                    handle: slider::Handle {
                        shape: slider::HandleShape::Rectangle { width: 0, border_radius: 0.0.into() },
                        background: iced::Background::Color(Color::TRANSPARENT),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }),
            
            button(
                container(
                    text(volume_icon)
                        .color(theme.color1)
                        .font(font)
                        .size(font_size * 1.6)
                        .center()
                )
                .width(Length::Fill)
                .height(Length::Fixed(15.0))
                .center_x(Length::Fill) 
            )
            .on_press(Message::AudioMuteToggle)
            .style(move |_, _| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border { color: theme.color2, width: 1.5, radius: 0.0.into() },
                ..Default::default()
            }),
        ]
        .spacing(5)
        .align_x(iced::alignment::Horizontal::Center);

        let brightness_column = column![
            container(
                text(format!("{}%", self.brightness_value as i32))
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(iced::padding::top(6).bottom(4)),
            
            vertical_slider(0.0..=100.0, self.brightness_value, Message::BrightnessChanged)
                .height(Length::Fixed(self.slider_height))
                .width(20.0)
                .step(1.0)
                .style(move |_theme, _status| slider::Style {
                    rail: slider::Rail {
                        backgrounds: (
                            iced::Background::Color(theme.color4),
                            iced::Background::Color(Color::from_rgba(theme.color6.r, theme.color6.g, theme.color6.b, 0.3)),
                        ),
                        width: 20.0,
                        border: Border { radius: 0.0.into(), ..Default::default() },
                    },
                    handle: slider::Handle {
                        shape: slider::HandleShape::Rectangle { width: 0, border_radius: 0.0.into() },
                        background: iced::Background::Color(Color::TRANSPARENT),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }),
            
            container(
                button(
                        container(
                            text(brightness_icon)
                                .color(theme.color1)
                                .font(font)
                                .size(font_size * 1.6)
                                .center()
                        )
                        .width(Length::Fill)
                        .height(Length::Fixed(15.0))
                        .center_x(Length::Fill) 
                    )
                    .on_press(Message::BrightnessMinToggle)
                    .style(move |_, _| button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border { color: theme.color2, width: 1.5, radius: 0.0.into() },
                        ..Default::default()
                    }),
                )
        ]
        .spacing(5)
        .align_x(iced::alignment::Horizontal::Center);

        // --- FINAL ASSEMBLY ---
        let sliders_row = row![volume_column, brightness_column]
            .spacing(20)
            .padding(iced::padding::right(1))
            .align_y(iced::alignment::Vertical::Center);

        let right_part = container(sliders_row)
            .width(Length::Fixed(70.0))
            .height(Length::Fill);

        let main_row = row![left_part, right_part].spacing(0);

        container(
            container(
                stack![
                    container(
                        container(
                            container(main_row)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(move |_| container::Style { background: None, ..Default::default() })
                        )
                        .padding(iced::padding::top(5).right(25).bottom(8).left(11))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        // FIX: Changed |_,_| to |_|
                        .style(move |_| container::Style {
                            background: None,
                            border: Border { color: theme.color3, width: 2.0, radius: 0.0.into() },
                            ..Default::default()
                        })
                    )
                    .padding(iced::padding::top(15))
                    .width(Length::Fill)
                    .height(Length::Fill),
                    
                    container(
                        container(
                            text(" Services ")
                                .color(theme.color6)
                                .font(font)
                                .size(font_size)
                        )
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        // FIX: Changed |_,_| to |_|
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
            .style(move |_| container::Style { background: None, ..Default::default() }),
        )
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .style(move |_| container::Style { background: None, ..Default::default() })
        .into()
    }

    pub fn set_slider_height(&mut self, height: f32) {
        self.slider_height = height;
    }

    pub fn set_volume(&mut self, value: f32) {
        self.volume_value = value.clamp(0.0, 100.0);
        if self.volume_value > 0.0 {
            self.is_muted = false;
        }
        let _ = Command::new("pactl")
            .arg("set-sink-volume")
            .arg("@DEFAULT_SINK@")
            .arg(format!("{}%", self.volume_value as u8))
            .output();
    }

    pub fn set_brightness(&mut self, value: f32) {
        self.brightness_value = value.clamp(0.0, 100.0);
        if self.brightness_value > 0.0 {
            self.is_min_brightness = false;
        }
        let _ = Command::new("brightnessctl")
            .arg("s")
            .arg(format!("{}%", self.brightness_value as u8))
            .output();
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
        if self.is_muted {
            self.previous_volume_value = self.volume_value;
            self.set_volume(0.0);
        } else {
            self.set_volume(self.previous_volume_value);
        }
    }

    pub fn toggle_min_brightness(&mut self) {
        self.is_min_brightness = !self.is_min_brightness;
        if self.is_min_brightness {
            self.previous_brightness_value = self.brightness_value;
            self.set_brightness(0.0);
        } else {
            self.set_brightness(self.previous_brightness_value);
        }
    }
}