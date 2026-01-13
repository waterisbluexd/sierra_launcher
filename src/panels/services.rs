use iced::widget::{container, text, stack, row, column, vertical_slider, slider};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;

pub struct ServicesPanel {
    pub volume_value: f32,
    pub brightness_value: f32,
    pub slider_height: f32,
}

impl ServicesPanel {
    pub fn new() -> Self {
        Self {
            volume_value: 50.0,
            brightness_value: 50.0,
            slider_height: 200.0,
        }
    }

    pub fn view<'a>(
        &self,
        theme: &'a Theme,
        bg_with_alpha: Color,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        // Create volume slider column
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
            .padding(5),
            
            // Vertical slider with custom style
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
                            iced::Background::Color(theme.color4), // Filled portion
                            iced::Background::Color(Color::from_rgba(
                                theme.color6.r,
                                theme.color6.g,
                                theme.color6.b,
                                0.0
                            )), // Unfilled portion
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
            
            // Icon/Label
            container(
                text("🔊")
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(5),
        ]
        .spacing(10)
        .align_x(iced::alignment::Horizontal::Center);

        // Create brightness slider column
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
            .padding(5),
            
            // Vertical slider with custom style
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
                            iced::Background::Color(theme.color4), // Filled portion
                            iced::Background::Color(Color::from_rgba(
                                theme.color6.r,
                                theme.color6.g,
                                theme.color6.b,
                                0.0
                            )), // Unfilled portion
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
            
            // Icon/Label
            container(
                text("☀")
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(5),
        ]
        .spacing(10)
        .align_x(iced::alignment::Horizontal::Center);

        // Combine both sliders in a row
        let sliders_row = row![
            volume_column,
            brightness_column
        ]
        .spacing(30)
        .padding(10);

        container(
            container(
                stack![
                    container(
                        container(
                            container(
                                sliders_row
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .style(move |_| container::Style {
                                background: None,
                                ..Default::default()
                            })
                        )
                        .padding(10)
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
    }

    pub fn set_brightness(&mut self, value: f32) {
        self.brightness_value = value.clamp(0.0, 100.0);
    }
}