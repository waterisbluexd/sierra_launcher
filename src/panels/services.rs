use iced::widget::{container, text, stack, row, column, vertical_slider, slider, button};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;
use std::process::Command;
use regex::Regex;

pub struct ServicesPanel {
    pub volume_value: f32,
    pub brightness_value: f32,
    pub slider_height: f32,
}

impl ServicesPanel {
    pub fn new() -> Self {
        let volume_value = Self::get_volume().unwrap_or(50.0);
        let brightness_value = Self::get_brightness().unwrap_or(50.0);

        Self {
            volume_value,
            brightness_value,
            slider_height: 120.0,  // Reduced from 200.0
        }
    }

    fn get_volume() -> Option<f32> {
        let output = Command::new("amixer").arg("sget").arg("Master").output().ok()?;
        let output_str = String::from_utf8(output.stdout).ok()?;
        let re = Regex::new(r"\[(\d+)%\]").unwrap();
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

    pub fn view<'a>(
        &self,
        theme: &'a Theme,
        bg_with_alpha: Color,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        // Left part - empty for now
        let left_part = container(
            text("")
        )
        .width(Length::Fill)
        .height(Length::Fill);

        let volume_icon = if self.volume_value == 0.0 {
            ""
        } else if self.volume_value <= 33.0 {
            ""
        } else if self.volume_value <= 66.0 {
            ""
        } else {
            ""
        };

        let volume_column = column![
            // Percentage value
            container(
                text(format!("{}%", self.volume_value as i32))
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(iced::padding::top(6).bottom(4)),  // Reduced from 5
            
            // Vertical slider
            vertical_slider(
                0.0..=100.0,
                self.volume_value,
                Message::VolumeChanged
            )
            .height(Length::Fixed(self.slider_height))
            .width(20.0)
            .step(1.0)
            .style(move |_theme_palette, _status| {
                slider::Style {
                    rail: slider::Rail {
                        backgrounds: (
                            iced::Background::Color(theme.color4),
                            iced::Background::Color(Color::from_rgba(
                                theme.color6.r,
                                theme.color6.g,
                                theme.color6.b,
                                0.3
                            )),
                        ),
                        width: 20.0,
                        border: Border {
                            radius: 0.0.into(),
                            ..Default::default()
                        },
                    },
                    handle: slider::Handle {
                        shape: slider::HandleShape::Rectangle {
                            width: 0,
                            border_radius: 0.0.into(),
                        },
                        background: iced::Background::Color(Color::TRANSPARENT),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }),
            
            // Icon
            button(
                container(
                    text(volume_icon)
                        .color(theme.color2)
                        .font(font)
                        .size(font_size * 1.6)
                        .center()
                )
                .width(Length::Fill)
                .height(Length::Fixed(15.0))
                .center_x(Length::Fill) 
            )
            .on_press(Message::MusicPlayPause)
            .style(move |_, _| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: theme.color2,
                    width: 1.5,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
        ]
        .spacing(5)
        .align_x(iced::alignment::Horizontal::Center);

        // Create brightness slider column with label
        let brightness_column = column![
            // Percentage value
            container(
                text(format!("{}%", self.brightness_value as i32))
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(iced::padding::top(6).bottom(4)),  // Reduced from 5
            
            // Vertical slider
            vertical_slider(
                0.0..=100.0,
                self.brightness_value,
                Message::BrightnessChanged
            )
            .height(Length::Fixed(self.slider_height))
            .width(20.0)
            .step(1.0)
            .style(move |_theme_palette, _status| {
                slider::Style {
                    rail: slider::Rail {
                        backgrounds: (
                            iced::Background::Color(theme.color4),
                            iced::Background::Color(Color::from_rgba(
                                theme.color6.r,
                                theme.color6.g,
                                theme.color6.b,
                                0.3
                            )),
                        ),
                        width: 20.0,
                        border: Border {
                            radius: 0.0.into(),
                            ..Default::default()
                        },
                    },
                    handle: slider::Handle {
                        shape: slider::HandleShape::Rectangle {
                            width: 0,
                            border_radius: 0.0.into(),
                        },
                        background: iced::Background::Color(Color::TRANSPARENT),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }),
            
            // Icon
            container(
                button(
                        container(
                            text("")
                                .color(theme.color2)
                                .font(font)
                                .size(font_size * 1.6)
                                .center()
                        )
                        .width(Length::Fill)
                        .height(Length::Fixed(15.0))
                        .center_x(Length::Fill) 
                    )
                    .on_press(Message::MusicPlayPause)
                    .style(move |_, _| button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border {
                            color: theme.color2,
                            width: 1.5,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    }),
                )
        ]
        .spacing(5)
        .align_x(iced::alignment::Horizontal::Center);

        // Right part - sliders row
        let sliders_row = row![
            volume_column,
            brightness_column
        ]
        .spacing(20)
        .padding(iced::padding::right(1))  // Removed top(3)
        .align_y(iced::alignment::Vertical::Center);

        let right_part = container(sliders_row)
            .width(Length::Fixed(70.0))
            .height(Length::Fill);

        // Main content row with left and right parts
        let main_row = row![
            left_part,
            right_part
        ]
        .spacing(0);

        container(
            container(
                stack![
                    container(
                        container(
                            container(
                                main_row
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(move |_| container::Style {
                                background: None,
                                ..Default::default()
                            })
                        )
                        .padding(iced::padding::top(5).right(18).bottom(8))  // Changed from bottom(25) to bottom(8)
                        .width(Length::Fill)
                        .height(Length::Fill)
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
            .style(move |_| container::Style {
                background: None,
                ..Default::default()
            }),
        )
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .style(move |_| container::Style {
            background: None,
            ..Default::default()
        })
        .into()
    }

    pub fn set_slider_height(&mut self, height: f32) {
        self.slider_height = height;
    }

    pub fn set_volume(&mut self, value: f32) {
        self.volume_value = value.clamp(0.0, 100.0);
        let _ = Command::new("amixer")
            .arg("-q")
            .arg("sset")
            .arg("Master")
            .arg(format!("{}%", self.volume_value as u8))
            .output();
    }

    pub fn set_brightness(&mut self, value: f32) {
        self.brightness_value = value.clamp(0.0, 100.0);
        let _ = Command::new("brightnessctl")
            .arg("s")
            .arg(format!("{}%", self.brightness_value as u8))
            .output();
    }
}