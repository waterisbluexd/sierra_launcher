use iced::widget::{container, text, column, stack, row, Space};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;

pub fn right_main_panels_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
) -> Element<'a, Message> {
    container(
        column![
            // ──────────────────────────────
            // Panel 1
            // ──────────────────────────────
            container(text("Panel 1"))
                .width(Length::Fill)
                .height(Length::FillPortion(1))
                .style(move |_| container::Style {
                    background: Some(Color::from_rgb(0.2, 0.3, 0.4).into()),
                    ..Default::default()
                }),
            
            // ──────────────────────────────
            // Panel 2
            // ──────────────────────────────
            container(text("Panel 2"))
                .width(Length::Fill)
                .height(Length::FillPortion(2))
                .style(move |_| container::Style {
                    background: Some(Color::from_rgb(0.3, 0.4, 0.5).into()),
                    ..Default::default()
                }),
            
            // ──────────────────────────────
            // Panel 3 (stacked layers)
            // ──────────────────────────────
            container(
                stack![
                    // Background layer (bottom) - the bordered input area and PowerOff button
                    container(
                        row![
                            container(text(""))
                                .width(Length::FillPortion(1))
                                .height(Length::Fixed(35.0))
                                .style(move |_| container::Style {
                                    background: Some(bg_with_alpha.into()),
                                    border: Border {
                                        color: theme.border,
                                        width: 2.0,
                                        radius: 0.0.into(),
                                    },
                                    ..Default::default()
                                }),
                            
                            container(text("PowerOff"))
                                .width(Length::Fixed(35.0))
                                .height(Length::Fill)
                                .style(move |_| container::Style {
                                    background: Some(Color::from_rgb(0.5, 0.6, 0.7).into()),
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
                    
                    // Foreground layer (top) - the "Input" text
                    container(
                        container(text(" Input "))
                            .padding(0)
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .style(move |_| container::Style {
                                background: Some(bg_with_alpha.into()),
                                ..Default::default()
                            })
                    )
                    .padding(iced::padding::bottom(30).left(15))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_| container::Style {
                        background: None, // Changed from bg_with_alpha so it doesn't cover the border
                        ..Default::default()
                    }),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fixed(45.0))
            .style(move |_| container::Style {
                background: Some(bg_with_alpha.into()),
                ..Default::default()
            }),
        ]
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(bg_with_alpha.into()),
        border: Border {
            color: theme.border,
            width: 2.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}