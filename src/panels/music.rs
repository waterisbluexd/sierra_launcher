use iced::widget::{container, text, stack};
use iced::{Element, Border, Color, Length};

use crate::utils::theme::Theme;
use crate::Message;

pub fn music_panel_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    container(
        container(
            stack![
                // Background + content container
                container(
                    container(text(""))
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
                        text(" Music ")
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