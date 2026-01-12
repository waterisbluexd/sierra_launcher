use iced::widget::{container, text, stack, row, column, button, slider};
use iced::{Element, Border, Color, Length, Alignment};
use crate::utils::theme::Theme;
use crate::Message;
use crate::mpris_player_state::MusicPlayerState;

pub fn music_panel_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: iced::Font,
    font_size: f32,
    music_state: &'a MusicPlayerState,
) -> Element<'a, Message> {
    let play_pause_icon = if music_state.is_playing { "⏸" } else { "▶" };
    
    container(
        container(
            stack![
                container(
                    container(
                        if music_state.player_available {
                            // Player is available - show music controls
                            column![
                                // Application name at the top
                                container(
                                    text(&music_state.app_name)
                                        .color(Color::from_rgba(
                                            theme.color6.r,
                                            theme.color6.g,
                                            theme.color6.b,
                                            0.6
                                        ))
                                        .font(font)
                                        .size(font_size * 0.7)
                                )
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .padding(iced::padding::bottom(20)),
                                
                                // Song name (big)
                                container(
                                    text(&music_state.song_name)
                                        .color(theme.color6)
                                        .font(font)
                                        .size(font_size * 1.2)
                                )
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .padding(iced::padding::bottom(5)),
                                
                                // Artist name (small)
                                container(
                                    text(&music_state.artist_name)
                                        .color(Color::from_rgba(
                                            theme.color6.r,
                                            theme.color6.g,
                                            theme.color6.b,
                                            0.7
                                        ))
                                        .font(font)
                                        .size(font_size * 0.8)
                                )
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .padding(iced::padding::bottom(30)),
                                
                                // Progress bar with time stamps
                                column![
                                    // Time labels
                                    row![
                                        text(MusicPlayerState::format_time(music_state.current_time))
                                            .color(theme.color6)
                                            .font(font)
                                            .size(font_size * 0.7),
                                        container(text(""))
                                            .width(Length::Fill),
                                        text(MusicPlayerState::format_time(music_state.total_time))
                                            .color(theme.color6)
                                            .font(font)
                                            .size(font_size * 0.7),
                                    ]
                                    .width(Length::Fill)
                                    .padding(iced::padding::bottom(5)),
                                    
                                    // Progress slider
                                    slider(
                                        0.0..=music_state.total_time.max(1.0),
                                        music_state.current_time,
                                        Message::MusicProgressChanged
                                    )
                                    .width(Length::Fill)
                                    .step(1.0)
                                ]
                                .width(Length::Fill)
                                .padding(iced::padding::bottom(25).left(10).right(10)),
                                
                                // Control buttons
                                row![
                                    // Previous button
                                    button(
                                        container(
                                            text("⏮")
                                                .color(theme.color6)
                                                .font(font)
                                                .size(font_size * 1.2)
                                        )
                                        .width(Length::Fixed(50.0))
                                        .height(Length::Fixed(50.0))
                                        .center_x(Length::Fill)
                                        .center_y(Length::Fill)
                                    )
                                    .on_press(Message::MusicPrevious)
                                    .style(move |_, _| button::Style {
                                        background: Some(Color::TRANSPARENT.into()),
                                        border: Border {
                                            color: theme.color3,
                                            width: 1.5,
                                            radius: 25.0.into(),
                                        },
                                        ..Default::default()
                                    }),
                                    
                                    // Spacer
                                    container(text(""))
                                        .width(Length::Fixed(30.0)),
                                    
                                    // Play/Pause button
                                    button(
                                        container(
                                            text(play_pause_icon)
                                                .color(theme.color6)
                                                .font(font)
                                                .size(font_size * 1.5)
                                        )
                                        .width(Length::Fixed(60.0))
                                        .height(Length::Fixed(60.0))
                                        .center_x(Length::Fill)
                                        .center_y(Length::Fill)
                                    )
                                    .on_press(Message::MusicPlayPause)
                                    .style(move |_, _| button::Style {
                                        background: Some(Color::TRANSPARENT.into()),
                                        border: Border {
                                            color: theme.color3,
                                            width: 2.0,
                                            radius: 30.0.into(),
                                        },
                                        ..Default::default()
                                    }),
                                    
                                    // Spacer
                                    container(text(""))
                                        .width(Length::Fixed(30.0)),
                                    
                                    // Next button
                                    button(
                                        container(
                                            text("⏭")
                                                .color(theme.color6)
                                                .font(font)
                                                .size(font_size * 1.2)
                                        )
                                        .width(Length::Fixed(50.0))
                                        .height(Length::Fixed(50.0))
                                        .center_x(Length::Fill)
                                        .center_y(Length::Fill)
                                    )
                                    .on_press(Message::MusicNext)
                                    .style(move |_, _| button::Style {
                                        background: Some(Color::TRANSPARENT.into()),
                                        border: Border {
                                            color: theme.color3,
                                            width: 1.5,
                                            radius: 25.0.into(),
                                        },
                                        ..Default::default()
                                    }),
                                ]
                                .width(Length::Fill)
                                .align_y(Alignment::Center)
                                .spacing(0)
                                .padding(iced::padding::left(0).right(0))
                            ]
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                        } else {
                            // No player available - show placeholder
                            column![
                                container(text(""))
                                    .height(Length::Fill),
                                
                                container(
                                    text("No Music Playing")
                                        .color(theme.color6)
                                        .font(font)
                                        .size(font_size * 1.0)
                                )
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .padding(iced::padding::bottom(10)),
                                
                                container(
                                    text("Start playing music in Spotify, YouTube,")
                                        .color(Color::from_rgba(
                                            theme.color6.r,
                                            theme.color6.g,
                                            theme.color6.b,
                                            0.5
                                        ))
                                        .font(font)
                                        .size(font_size * 0.7)
                                )
                                .width(Length::Fill)
                                .center_x(Length::Fill)
                                .padding(iced::padding::bottom(5)),
                                
                                container(
                                    text("or any MPRIS-compatible player")
                                        .color(Color::from_rgba(
                                            theme.color6.r,
                                            theme.color6.g,
                                            theme.color6.b,
                                            0.5
                                        ))
                                        .font(font)
                                        .size(font_size * 0.7)
                                )
                                .width(Length::Fill)
                                .center_x(Length::Fill),
                                
                                container(text(""))
                                    .height(Length::Fill),
                            ]
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Alignment::Center)
                        }
                    )
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