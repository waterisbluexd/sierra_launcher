use iced::widget::{container, text, column, stack, row};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;
use crate::panels::search_bar::SearchBar;
use crate::panels::app_list::AppList;
use crate::panels::clock;
use crate::panels::weather;


use crate::Panel;

pub fn right_main_panels_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: iced::Font,
    font_size: f32,
    search_bar: &'a SearchBar,
    app_list: &'a AppList,
    current_panel: crate::Panel,
    weather_panel: &'a weather::WeatherPanel,
) -> Element<'a, Message> {
    let current_view = match current_panel {
        Panel::Clock => clock::clock_panel_view(theme, bg_with_alpha, font, font_size),
        Panel::Weather => weather_panel.view(theme, bg_with_alpha, font, font_size),
    };
    
    container(
        column![
            // ──────────────────────────────
            // Panel 1 - Clock or Weather
            // ──────────────────────────────
            current_view,
            // ──────────────────────────────
            // Panel 2 (FIXED)
            // ──────────────────────────────
            container(
                stack![
                    container(
                        container(
                            container(
                                app_list.view(theme, font, font_size).map(Message::AppListMessage)
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .padding(iced::padding::top(15).right(15).left(15))
                            .style(move |_| container::Style {
                                background: None,
                                ..Default::default()
                            }),
                        )
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
                                .color(theme.color6)
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
                            // Search Bar container this container will allow user to add input like text 
                            container(
                                search_bar.view(theme, font, font_size).map(Message::SearchBarMessage)
                            )
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
                                    color: theme.color1,
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