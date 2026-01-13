use iced::widget::{container, text, column, stack};
use iced::{Element, Border, Color, Length, alignment};
use crate::utils::theme::Theme;
use crate::Message;

pub struct ServicesPanel;

impl ServicesPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn view<'a>(
        &self,
        theme: &'a Theme,
        bg_with_alpha: Color,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        let content = column![
            text("Services Panel")
                .color(theme.color6)
                .font(font)
                .size(font_size * 2.0)
                .line_height(1.3),
        ]
        .spacing(20)
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill);

        container(
            container(
                stack![
                    // Background + content container
                    container(
                        container(content)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .padding(iced::padding::top(25))
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
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
                    // Floating title label
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
}