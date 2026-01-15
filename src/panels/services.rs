use iced::widget::{container, text, stack, row, column, vertical_slider, slider, button};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;
use std::process::Command;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct ServiceStatus {
    wifi_enabled: bool,
    wifi_name: String,
    bluetooth_enabled: bool,
    bluetooth_name: String,
    last_update: Instant,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self {
            wifi_enabled: false,
            wifi_name: "Checking...".to_string(),
            bluetooth_enabled: false,
            bluetooth_name: "Checking...".to_string(),
            last_update: Instant::now() - Duration::from_secs(10), // Force initial update
        }
    }
}

pub struct ServicesPanel {
    pub volume_value: f32,
    pub brightness_value: f32,
    pub slider_height: f32,
    previous_volume_value: f32,
    is_muted: bool,
    previous_brightness_value: f32,
    is_min_brightness: bool,
    pub is_airplane_mode_on: bool,
    pub eye_care_enabled: bool,
    
    // Cache status with Arc<Mutex> for thread-safe access
    status_cache: Arc<Mutex<ServiceStatus>>,
    refresh_requested: Arc<Mutex<bool>>,
}

impl ServicesPanel {
    pub fn new() -> Self {
        let volume_value = Self::get_volume().unwrap_or(50.0);
        let brightness_value = Self::get_brightness().unwrap_or(50.0);
        
        let status_cache = Arc::new(Mutex::new(ServiceStatus::default()));
        let refresh_requested = Arc::new(Mutex::new(true));
        
        // Start background thread for status updates
        let cache_clone = Arc::clone(&status_cache);
        let refresh_clone = Arc::clone(&refresh_requested);
        
        std::thread::spawn(move || {
            loop {
                // Check if refresh is requested
                let should_refresh = {
                    let mut requested = refresh_clone.lock().unwrap();
                    if *requested {
                        *requested = false;
                        true
                    } else {
                        false
                    }
                };
                
                if should_refresh {
                    // Fetch status in background thread (non-blocking)
                    let (wifi_enabled, wifi_name) = Self::fetch_wifi_status();
                    let (bt_enabled, bt_name) = Self::fetch_bluetooth_status();
                    
                    // Update cache
                    if let Ok(mut status) = cache_clone.lock() {
                        status.wifi_enabled = wifi_enabled;
                        status.wifi_name = wifi_name;
                        status.bluetooth_enabled = bt_enabled;
                        status.bluetooth_name = bt_name;
                        status.last_update = Instant::now();
                    }
                }
                
                // Sleep to prevent busy-waiting
                std::thread::sleep(Duration::from_millis(200));
            }
        });

        Self {
            volume_value,
            brightness_value,
            slider_height: 120.0,
            previous_volume_value: volume_value,
            is_muted: false,
            previous_brightness_value: brightness_value,
            is_min_brightness: false,
            is_airplane_mode_on: false,
            eye_care_enabled: false,
            status_cache,
            refresh_requested,
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

    fn fetch_wifi_status() -> (bool, String) {
        // Try nmcli first (NetworkManager) with timeout
        if let Ok(output) = Command::new("timeout")
            .args(&["1", "nmcli", "-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
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

        // Fallback to iwgetid with timeout
        if let Ok(output) = Command::new("timeout")
            .args(&["1", "iwgetid", "-r"])
            .output() {
            if let Ok(ssid) = String::from_utf8(output.stdout) {
                let ssid = ssid.trim();
                if !ssid.is_empty() {
                    return (true, ssid.to_string());
                }
            }
        }

        // Check if WiFi is disabled
        if let Ok(output) = Command::new("timeout")
            .args(&["1", "nmcli", "radio", "wifi"])
            .output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if stdout.trim() == "disabled" {
                    return (false, "WiFi Off".to_string());
                }
            }
        }

        // WiFi is on but not connected
        (true, "No Network".to_string())
    }

    fn fetch_bluetooth_status() -> (bool, String) {
        // Check if bluetooth is powered on using bluetoothctl with timeout
        if let Ok(output) = Command::new("timeout")
            .args(&["1", "bluetoothctl", "show"])
            .output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                let powered = stdout.lines()
                    .find(|line| line.contains("Powered:"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|s| s.trim() == "yes")
                    .unwrap_or(false);

                if !powered {
                    return (false, "Bluetooth Off".to_string());
                }

                // Check for connected devices with timeout
                if let Ok(devices_output) = Command::new("timeout")
                    .args(&["1", "bluetoothctl", "devices", "Connected"])
                    .output() {
                    if let Ok(devices_str) = String::from_utf8(devices_output.stdout) {
                        if let Some(first_device) = devices_str.lines().next() {
                            // Extract device name (format: "Device MAC_ADDRESS Name")
                            let parts: Vec<&str> = first_device.split_whitespace().collect();
                            if parts.len() >= 3 {
                                let name = parts[2..].join(" ");
                                return (true, name);
                            }
                        }
                    }
                }

                // Bluetooth is on but no device connected
                return (true, "No Device".to_string());
            }
        }

        // Fallback - assume bluetooth is available but off
        (false, "Bluetooth Off".to_string())
    }

    pub fn schedule_refresh(&self) {
        if let Ok(mut refresh) = self.refresh_requested.lock() {
            *refresh = true;
        }
    }

    pub fn toggle_wifi(&mut self) {
        // Get current state from cache
        let is_enabling = !self.wifi_enabled();
        
        std::thread::spawn(move || {
            if is_enabling {
                let _ = Command::new("nmcli").args(&["radio", "wifi", "on"]).output();
            } else {
                let _ = Command::new("nmcli").args(&["radio", "wifi", "off"]).output();
            }
        });
        
        // Update cache immediately for responsiveness
        if let Ok(mut status) = self.status_cache.lock() {
            status.wifi_enabled = is_enabling;
            if !is_enabling {
                status.wifi_name = "WiFi Off".to_string();
            }
        }
        
        // Schedule refresh after a delay
        let refresh_clone = Arc::clone(&self.refresh_requested);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            if let Ok(mut refresh) = refresh_clone.lock() {
                *refresh = true;
            }
        });
    }

    pub fn toggle_bluetooth(&mut self) {
        // Get current state from cache
        let is_enabling = !self.bluetooth_enabled();
        
        std::thread::spawn(move || {
            if is_enabling {
                let _ = Command::new("bluetoothctl").args(&["power", "on"]).output();
            } else {
                let _ = Command::new("bluetoothctl").args(&["power", "off"]).output();
            }
        });
        
        // Update cache immediately for responsiveness
        if let Ok(mut status) = self.status_cache.lock() {
            status.bluetooth_enabled = is_enabling;
            if !is_enabling {
                status.bluetooth_name = "Bluetooth Off".to_string();
            }
        }
        
        // Schedule refresh after a delay
        let refresh_clone = Arc::clone(&self.refresh_requested);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
            if let Ok(mut refresh) = refresh_clone.lock() {
                *refresh = true;
            }
        });
    }

    pub fn toggle_eye_care(&mut self) {
        self.eye_care_enabled = !self.eye_care_enabled;
        
        let is_enabled = self.eye_care_enabled;
        std::thread::spawn(move || {
            if is_enabled {
                let _ = Command::new("redshift").args(&["-P", "-O", "3500"]).output();
            } else {
                let _ = Command::new("redshift").args(&["-x"]).output();
            }
        });
    }

    // Getter methods to access cached values without blocking
    pub fn wifi_enabled(&self) -> bool {
        self.status_cache.lock().map(|s| s.wifi_enabled).unwrap_or(false)
    }

    pub fn wifi_name(&self) -> String {
        self.status_cache.lock()
            .map(|s| s.wifi_name.clone())
            .unwrap_or_else(|_| "Unknown".to_string())
    }

    pub fn bluetooth_enabled(&self) -> bool {
        self.status_cache.lock().map(|s| s.bluetooth_enabled).unwrap_or(false)
    }

    pub fn bluetooth_name(&self) -> String {
        self.status_cache.lock()
            .map(|s| s.bluetooth_name.clone())
            .unwrap_or_else(|_| "Unknown".to_string())
    }

    pub fn view<'a>(
        &'a self,
        theme: &'a Theme,
        bg_with_alpha: Color,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        
        // Get cached values (non-blocking)
        let wifi_enabled = self.wifi_enabled();
        let wifi_name = self.wifi_name();
        let bt_enabled = self.bluetooth_enabled();
        let bt_name = self.bluetooth_name();
        
        // --- 1. DETERMINE WIFI STYLING COLORS ---
        let is_connected = wifi_enabled && wifi_name != "No Network" && wifi_name != "WiFi Off";
        
        let active_accent = if is_connected { theme.color2 } else { theme.color3 };
        let inactive_accent = theme.color8;

        let (wifi_text_color, _wifi_bg_color, _wifi_border_color) = if wifi_enabled {
            (theme.color0, active_accent, active_accent)
        } else {
            (inactive_accent, Color::TRANSPARENT, inactive_accent)
        };

        let wifi_icon_str = if wifi_enabled { "󰤨" } else { "󰤮" };

        // --- BLUETOOTH STYLING COLORS ---
        let is_bt_connected = bt_enabled && bt_name != "No Device" && bt_name != "Bluetooth Off";
        
        let bt_active_accent = if is_bt_connected { theme.color2 } else { theme.color3 };

        let (bt_text_color, _bt_bg_color, _bt_border_color) = if bt_enabled {
            (theme.color0, bt_active_accent, bt_active_accent)
        } else {
            (inactive_accent, Color::TRANSPARENT, inactive_accent)
        };

        let bt_icon_str = if bt_enabled { "" } else { "󰂲" };

        // --- 2. BUILD THE WIFI BUTTON CONTENT ---
        let wifi_button_content = container(
            row![
                container(
                    text(wifi_icon_str)
                        .color(wifi_text_color)
                        .font(font)
                        .size(font_size * 2.2)
                        .center()
                )
                .padding(iced::padding::right(12))
                .align_y(iced::alignment::Vertical::Center),

                column![
                    text("CONNECTION")
                        .color(wifi_text_color)
                        .size(font_size * 0.65)
                        .font(font),
                    text(if wifi_name.len() > 14 { 
                        format!("{}..", &wifi_name[..12]) 
                    } else { 
                        wifi_name.clone() 
                    })
                        .color(wifi_text_color)
                        .size(font_size * 0.9)
                        .font(font),
                ]
                .spacing(2)
                .align_x(iced::alignment::Horizontal::Left)
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::padding::left(15).right(5))
        .align_y(iced::alignment::Vertical::Center);

        // --- BLUETOOTH BUTTON CONTENT ---
        let bt_button_content = container(
            row![
                container(
                    text(bt_icon_str)
                        .color(bt_text_color)
                        .font(font)
                        .size(font_size * 2.2)
                        .center()
                )
                .padding(iced::padding::right(12))
                .align_y(iced::alignment::Vertical::Center),

                column![
                    text("BLUETOOTH")
                        .color(bt_text_color)
                        .size(font_size * 0.65)
                        .font(font),
                    text(if bt_name.len() > 14 { 
                        format!("{}..", &bt_name[..12]) 
                    } else { 
                        bt_name.clone() 
                    })
                        .color(bt_text_color)
                        .size(font_size * 0.9)
                        .font(font),
                ]
                .spacing(2)
                .align_x(iced::alignment::Horizontal::Left)
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::padding::left(15).right(5))
        .align_y(iced::alignment::Vertical::Center);

        // --- Determine Airplane Mode Styling Colors ---
        let airplane_active_color = theme.color2;
        let airplane_inactive_color = theme.color8;

        let (airplane_text_color, airplane_bg_color, airplane_border_color) = if self.is_airplane_mode_on {
            (theme.color0, airplane_active_color, airplane_active_color)
        } else {
            (airplane_inactive_color, Color::TRANSPARENT, airplane_inactive_color)
        };

        // --- Eye Care Styling Colors ---
        let eye_care_active_color = theme.color2;
        let eye_care_inactive_color = theme.color8;

        let (eye_care_text_color, eye_care_bg_color, eye_care_border_color) = if self.eye_care_enabled {
            (theme.color0, eye_care_active_color, eye_care_active_color)
        } else {
            (eye_care_inactive_color, Color::TRANSPARENT, eye_care_inactive_color)
        };

        // --- 3. ASSEMBLE LEFT PANEL ---
        let left_part = container(
            column![
                // Top Row: WiFi + Airplane
                container(
                    row![
                        container(
                            button(wifi_button_content)
                                .on_press(if self.is_airplane_mode_on { Message::NoOp } else { Message::WifiToggle })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .style(move |_theme, status| {
                                    let current_active_accent = if is_connected { theme.color2 } else { theme.color3 };
                                    let (current_wifi_text_color, current_wifi_bg_color, current_wifi_border_color) = if wifi_enabled {
                                        (theme.color0, current_active_accent, current_active_accent)
                                    } else {
                                        (inactive_accent, Color::TRANSPARENT, inactive_accent)
                                    };

                                    if self.is_airplane_mode_on {
                                        button::Style {
                                            background: Some(Color::from_rgba(0.5, 0.5, 0.5, 0.1).into()),
                                            border: Border {
                                                color: Color::from_rgb(0.5, 0.5, 0.5),
                                                width: 1.5,
                                                radius: 0.0.into(),
                                            },
                                            text_color: Color::from_rgb(0.5, 0.5, 0.5),
                                            ..Default::default()
                                        }
                                    } else {
                                        match status {
                                            iced::widget::button::Status::Hovered => button::Style {
                                                background: Some(if wifi_enabled {
                                                    let mut c = current_wifi_bg_color; c.a = 0.9; c.into()
                                                } else {
                                                    let mut c = current_active_accent; c.a = 0.1; c.into()
                                                }),
                                                border: Border {
                                                    color: current_active_accent,
                                                    width: 2.0,
                                                    radius: 0.0.into(),
                                                },
                                                text_color: current_wifi_text_color,
                                                ..Default::default()
                                            },
                                            iced::widget::button::Status::Pressed => button::Style {
                                                background: Some(current_active_accent.into()),
                                                border: Border { color: current_active_accent, width: 2.0, radius: 0.0.into() },
                                                text_color: theme.color0,
                                                ..Default::default()
                                            },
                                            _ => button::Style {
                                                background: Some(current_wifi_bg_color.into()),
                                                border: Border {
                                                    color: current_wifi_border_color,
                                                    width: 1.5,
                                                    radius: 0.0.into(),
                                                },
                                                text_color: current_wifi_text_color,
                                                ..Default::default()
                                            }
                                        }
                                    }
                                }),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill),

                        // Airplane Button
                        container(
                            button(
                                container(
                                    text("󰀝")
                                        .color(airplane_text_color)  
                                        .font(font)
                                        .size(font_size * 2.0)
                                        .center()
                                )
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x(Length::Fill) 
                                .center_y(Length::Fill) 
                            )
                            .on_press(Message::AirplaneModeToggle)
                            .style(move |_theme, status| {
                                match status {
                                    iced::widget::button::Status::Hovered => button::Style {
                                        background: Some(if self.is_airplane_mode_on {
                                            let mut c = airplane_bg_color; c.a = 0.9; c.into()
                                        } else {
                                            let mut c = airplane_inactive_color; c.a = 0.1; c.into()
                                        }),
                                        border: Border {
                                            color: if self.is_airplane_mode_on { airplane_active_color } else { airplane_inactive_color },
                                            width: 2.0,
                                            radius: 0.0.into(),
                                        },
                                        text_color: airplane_text_color,
                                        ..Default::default()
                                    },
                                    iced::widget::button::Status::Pressed => button::Style {
                                        background: Some(airplane_active_color.into()),
                                        border: Border {
                                            color: airplane_active_color,
                                            width: 2.0,
                                            radius: 0.0.into(),
                                        },
                                        text_color: theme.color0,
                                        ..Default::default()
                                    },
                                    _ => button::Style {
                                        background: Some(airplane_bg_color.into()),
                                        border: Border {
                                            color: airplane_border_color,
                                            width: 1.5,
                                            radius: 0.0.into(),
                                        },
                                        text_color: airplane_text_color,
                                        ..Default::default()
                                    }
                                }
                            }),
                        )
                        .width(Length::Fixed(45.0))
                        .height(Length::Fill)
                    ].spacing(10)
                )
                .width(Length::Fill)
                .height(Length::Fixed(45.0)),

                // Middle Row: Bluetooth + Eye Care + Settings
                container(
                    row![
                        container(
                            button(bt_button_content)
                                .on_press(if self.is_airplane_mode_on { Message::NoOp } else { Message::BluetoothToggle })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .style(move |_theme, status| {
                                    let current_bt_active_accent = if is_bt_connected { theme.color2 } else { theme.color3 };
                                    let (current_bt_text_color, current_bt_bg_color, current_bt_border_color) = if bt_enabled {
                                        (theme.color0, current_bt_active_accent, current_bt_active_accent)
                                    } else {
                                        (inactive_accent, Color::TRANSPARENT, inactive_accent)
                                    };

                                    if self.is_airplane_mode_on {
                                        button::Style {
                                            background: Some(Color::from_rgba(0.5, 0.5, 0.5, 0.1).into()),
                                            border: Border {
                                                color: Color::from_rgb(0.5, 0.5, 0.5),
                                                width: 1.5,
                                                radius: 0.0.into(),
                                            },
                                            text_color: Color::from_rgb(0.5, 0.5, 0.5),
                                            ..Default::default()
                                        }
                                    } else {
                                        match status {
                                            iced::widget::button::Status::Hovered => button::Style {
                                                background: Some(if bt_enabled {
                                                    let mut c = current_bt_bg_color; c.a = 0.9; c.into()
                                                } else {
                                                    let mut c = current_bt_active_accent; c.a = 0.1; c.into()
                                                }),
                                                border: Border {
                                                    color: current_bt_active_accent,
                                                    width: 2.0,
                                                    radius: 0.0.into(),
                                                },
                                                text_color: current_bt_text_color,
                                                ..Default::default()
                                            },
                                            iced::widget::button::Status::Pressed => button::Style {
                                                background: Some(current_bt_active_accent.into()),
                                                border: Border { color: current_bt_active_accent, width: 2.0, radius: 0.0.into() },
                                                text_color: theme.color0,
                                                ..Default::default()
                                            },
                                            _ => button::Style {
                                                background: Some(current_bt_bg_color.into()),
                                                border: Border {
                                                    color: current_bt_border_color,
                                                    width: 1.5,
                                                    radius: 0.0.into(),
                                                },
                                                text_color: current_bt_text_color,
                                                ..Default::default()
                                            }
                                        }
                                    }
                                }),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill),

                        // Eye Care Button
                        container(
                            button(
                                container(
                                    text("󰈈")
                                        .color(eye_care_text_color)  
                                        .font(font)
                                        .size(font_size * 1.6)
                                        .center()
                                )
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x(Length::Fill) 
                                .center_y(Length::Fill) 
                            )
                            .on_press(Message::EyeCareToggle)
                            .style(move |_theme, status| {
                                match status {
                                    iced::widget::button::Status::Hovered => button::Style {
                                        background: Some(if self.eye_care_enabled {
                                            let mut c = eye_care_bg_color; c.a = 0.9; c.into()
                                        } else {
                                            let mut c = eye_care_inactive_color; c.a = 0.1; c.into()
                                        }),
                                        border: Border {
                                            color: if self.eye_care_enabled { eye_care_active_color } else { eye_care_inactive_color },
                                            width: 2.0,
                                            radius: 0.0.into(),
                                        },
                                        text_color: eye_care_text_color,
                                        ..Default::default()
                                    },
                                    iced::widget::button::Status::Pressed => button::Style {
                                        background: Some(eye_care_active_color.into()),
                                        border: Border {
                                            color: eye_care_active_color,
                                            width: 2.0,
                                            radius: 0.0.into(),
                                        },
                                        text_color: theme.color0,
                                        ..Default::default()
                                    },
                                    _ => button::Style {
                                        background: Some(eye_care_bg_color.into()),
                                        border: Border {
                                            color: eye_care_border_color,
                                            width: 1.5,
                                            radius: 0.0.into(),
                                        },
                                        text_color: eye_care_text_color,
                                        ..Default::default()
                                    }
                                }
                            }),
                        )
                        .width(Length::Fixed(45.0))
                        .height(Length::Fill),

                        // Settings Button (placeholder)
                        container(
                            button(
                                container(
                                    text("󰩮")
                                        .color(airplane_text_color)  
                                        .font(font)
                                        .size(font_size * 1.6)
                                        .center()
                                )
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x(Length::Fill) 
                                .center_y(Length::Fill) 
                            )
                            .on_press(Message::NoOp)
                            .style(move |_theme, status| {
                                match status {
                                    iced::widget::button::Status::Hovered => button::Style {
                                        background: Some({
                                            let mut c = airplane_inactive_color; c.a = 0.1; c.into()
                                        }),
                                        border: Border {
                                            color: airplane_inactive_color,
                                            width: 2.0,
                                            radius: 0.0.into(),
                                        },
                                        text_color: airplane_text_color,
                                        ..Default::default()
                                    },
                                    _ => button::Style {
                                        background: Some(Color::TRANSPARENT.into()),
                                        border: Border {
                                            color: airplane_inactive_color,
                                            width: 1.5,
                                            radius: 0.0.into(),
                                        },
                                        text_color: airplane_text_color,
                                        ..Default::default()
                                    }
                                }
                            }),
                        )
                        .width(Length::Fixed(45.0))
                        .height(Length::Fill)
                    ]
                    .spacing(10)
                )
                .width(Length::Fill)
                .height(Length::Fixed(45.0)),

                // Bottom Row (placeholder)
                container(
                    row![
                        container(text("y").color(theme.color3))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            .style(move |_| container::Style {
                                border: Border { color: theme.color3, width: 2.0, radius: 0.0.into() },
                                ..Default::default()
                            }),
                        container(text("y.2").color(theme.color3))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            .style(move |_| container::Style {
                                border: Border { color: theme.color3, width: 2.0, radius: 0.0.into() },
                                ..Default::default()
                            }),
                    ]
                    .spacing(10)
                )
                .width(Length::Fill)
                .height(Length::Fill), 
            ]
            .spacing(10)
        )
        .padding(iced::padding::top(10).bottom(3).right(12).left(5))
        .width(Length::Fill)
        .height(Length::Fill);

        // --- RIGHT PANEL (Sliders) ---
        let volume_icon = if self.is_muted || self.volume_value == 0.0 { "" } else if self.volume_value <= 33.0 { "" } else if self.volume_value <= 66.0 { "" } else { "" };
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
            .on_press(Message::VolumeMuteToggle)
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
                        .padding(iced::padding::top(5).right(25).bottom(8).left(5))
                        .width(Length::Fill)
                        .height(Length::Fill)
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
        
        // Spawn async to avoid blocking
        let vol = self.volume_value as u8;
        std::thread::spawn(move || {
            let _ = Command::new("pactl")
                .arg("set-sink-volume")
                .arg("@DEFAULT_SINK@")
                .arg(format!("{}%", vol))
                .output();
        });
    }

    pub fn set_brightness(&mut self, value: f32) {
        self.brightness_value = value.clamp(0.0, 100.0);
        if self.brightness_value > 0.0 {
            self.is_min_brightness = false;
        }
        
        // Spawn async to avoid blocking
        let bright = self.brightness_value as u8;
        std::thread::spawn(move || {
            let _ = Command::new("brightnessctl")
                .arg("s")
                .arg(format!("{}%", bright))
                .output();
        });
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

    pub fn toggle_airplane_mode(&mut self) {
        self.is_airplane_mode_on = !self.is_airplane_mode_on;
        if self.is_airplane_mode_on {
            if self.wifi_enabled() {
                self.toggle_wifi();
            }
            if self.bluetooth_enabled() {
                self.toggle_bluetooth();
            }
        } else {
            self.schedule_refresh();
        }
    }
}