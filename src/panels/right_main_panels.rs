use iced::widget::{container, text, column, stack, row};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;

pub fn right_main_panels_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    container(
        column![
            // ──────────────────────────────
            // Panel 1
            // ──────────────────────────────
            container(
                text("Panel 1")
                    .font(font)
                    .size(font_size)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Color::from_rgb(0.2, 0.3, 0.4).into()),
                ..Default::default()
            }),

            // ──────────────────────────────
            // Panel 2 (FIXED)
            // ──────────────────────────────
            container(
                stack![
                    container(
                        container(text(""))
                            .height(Length::Fill)
                            .width(Length::Fill)
                            .style(move |_| container::Style {
                                background: None,
                                border: Border {
                                    color: theme.color3,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            }),
                    )
                    .padding(iced::padding::top(9))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(move |_| container::Style {
                            background: None,
                            ..Default::default()
                        }),

                    container(
                        container(
                            text(" Apps ")
                                .font(font)
                                .size(font_size)
                        )
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .style(move |_| container::Style {
                            background: Some(bg_with_alpha.into()),
                            ..Default::default()
                        }),
                    )
                    .padding(iced::padding::left(8))
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .style(move |_| container::Style {
                        background: None,
                        ..Default::default()
                    }),
                ]
            )
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .style(move |_| container::Style {
                background: None,
                ..Default::default()
            }),

            // ──────────────────────────────
            // Panel 3 (stacked layers)
            // ──────────────────────────────
            container(
                stack![
                    container(
                        row![
                            container(text(""))
                                .width(Length::FillPortion(1))
                                .height(Length::Fixed(35.0))
                                .style(move |_| container::Style {
                                    background: Some(bg_with_alpha.into()),
                                    border: Border {
                                        color: theme.color6,
                                        width: 2.0,
                                        radius: 0.0.into(),
                                    },
                                    ..Default::default()
                                }),

                            container(
                                text(" ")  // POWER BUTTON HERE
                                    .font(font)
                                    .size(font_size)
                            )
                            .width(Length::Fixed(35.0))
                            .height(Length::Fill)
                            .style(move |_| container::Style {
                                background: None,
                                border: Border {
                                    color: theme.color6,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            }),
                        ]
                        .spacing(5)
                        .height(Length::Fill)
                    )
                    .padding(iced::padding::top(10))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(bg_with_alpha.into()),
                        ..Default::default()
                    }),

                    container(
                        container(
                            text(" Input ")
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
                    .padding(iced::padding::bottom(30).left(8))
                    .width(Length::Fill)
                    .height(Length::Fill)
                ]
            )
            .width(Length::Fill)
            .height(Length::Fixed(45.0))
            .style(move |_| container::Style {
                background: Some(bg_with_alpha.into()),
                ..Default::default()
            }),
        ]
        .spacing(5)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: None,
        ..Default::default()
    })
    .into()
}