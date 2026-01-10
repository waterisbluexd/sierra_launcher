mod utils;
mod config;

use iced::widget::{container, text, stack, row};
use iced::{Element, Event, Border, Color, Length, Font, Task as Command, event};
use iced_layershell::actions::LayershellCustomActionWithId;
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use crate::utils::theme::{Theme, WalColors};
use crate::utils::watcher::ColorWatcher;
use crate::config::Config;

fn main() -> Result<(), iced_layershell::Error> {
    application(
        Launcher::new,
        Launcher::namespace,
        Launcher::update,
        Launcher::view,
    )
    .settings(Settings {
        layer_settings: LayerShellSettings {
            size: Some((484, 714)),
            anchor: Anchor::Bottom,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            margin: (0, 0, 4, 0),
            ..Default::default()
        },
        ..Default::default()
    })
    .style(|_theme, _id| iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    })
    .subscription(Launcher::subscription)
    .run()?;

    Ok(())
}

/* ================================
   Vertical text helper
   ================================ */
fn vertical_text(s: &str) -> String {
    s.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

struct Launcher {
    theme: Theme,
    watcher: Option<ColorWatcher>,
    config: Config,
}

#[derive(Debug, Clone)]
enum Message {
    IcedEvent(Event),
    CheckColors,
}

impl TryInto<LayershellCustomActionWithId> for Message {
    type Error = Self;
    fn try_into(self) -> Result<LayershellCustomActionWithId, Self::Error> {
        Err(self)
    }
}

impl Launcher {
    fn new() -> (Self, Command<Message>) {
        let theme = WalColors::load()
            .map(|w| w.to_theme())
            .unwrap_or_else(|_| Theme {
                background: Color::from_rgba(0.15, 0.15, 0.18, 0.82),
                foreground: Color::WHITE,
                border: Color::from_rgb(0.5, 0.5, 0.5),
                accent: Color::from_rgb(0.6, 0.6, 0.6),
                color0: Color::BLACK,
                color1: Color::from_rgb(0.8, 0.0, 0.0),
                color2: Color::from_rgb(0.0, 0.8, 0.0),
                color3: Color::from_rgb(0.8, 0.8, 0.0),
                color4: Color::from_rgb(0.0, 0.0, 0.8),
                color5: Color::from_rgb(0.8, 0.0, 0.8),
                color6: Color::from_rgb(0.0, 0.8, 0.8),
                color7: Color::from_rgb(0.7, 0.7, 0.7),
                color8: Color::from_rgb(0.5, 0.5, 0.5),
                color9: Color::from_rgb(1.0, 0.0, 0.0),
                color10: Color::from_rgb(0.0, 1.0, 0.0),
                color11: Color::from_rgb(1.0, 1.0, 0.0),
                color12: Color::from_rgb(0.0, 0.0, 1.0),
                color13: Color::from_rgb(1.0, 0.0, 1.0),
                color14: Color::from_rgb(0.0, 1.0, 1.0),
                color15: Color::WHITE,
            });

        let watcher = ColorWatcher::new().ok();
        let config = Config::load();

        (Self { theme, watcher, config }, Command::none())
    }

    fn namespace() -> String {
        String::from("iced_launcher2")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::window;

        let events = event::listen().map(Message::IcedEvent);
        let frames = window::frames().map(|_| Message::CheckColors);

        iced::Subscription::batch(vec![events, frames])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        use iced::keyboard;
        use keyboard::key::Named;

        match message {
            Message::IcedEvent(event) => {
                if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                    if let keyboard::Key::Named(Named::Escape) = key {
                        std::process::exit(0);
                    }
                }
                Command::none()
            }
            Message::CheckColors => {
                if let Some(ref watcher) = self.watcher {
                    if watcher.check_for_changes() {
                        if let Ok(wal_colors) = WalColors::load() {
                            self.theme = wal_colors.to_theme();
                        }
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let bg = self.theme.background;
        let bg_with_alpha = Color::from_rgba(bg.r, bg.g, bg.b, 1.0);

        let font = match self.config.font.as_deref() {
            Some("Monocraft") => Font::with_name("Monocraft"),
            Some("Monospace") => Font::with_name("Monospace"),
            _ => Font::default(),
        };

        let font_size = self.config.font_size.unwrap_or(22.0);
        let title = vertical_text(" sierra-launcher ");

        container(
            stack![
                // =========================
                // Container 2
                // =========================
                container(
                    row![
                        // Container 4 (left)
                        container(text(""))
                            .padding(9)
                            .height(Length::Fill)
                            .width(Length::Shrink)
                            .style(move |_| container::Style {
                                background: Some(bg_with_alpha.into()),
                                border: Border {
                                    color: self.theme.color3,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            }),

                        // Container 7 (right)
                        container(text(""))
                            .padding(50)
                            .height(Length::Fill)
                            .width(Length::Fill)
                            .style(move |_| container::Style {
                                background: Some(bg_with_alpha.into()),
                                border: Border {
                                    color: self.theme.color3,
                                    width: 2.0,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            }),
                    ]
                    .spacing(4)
                )
                .padding(14)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(bg_with_alpha.into()),
                    ..Default::default()
                }),

                // =========================
                // Container 3 (title)
                // =========================
                container(
                    container(
                        container(
                            text(title)
                                .font(font)
                                .size(font_size)
                                .line_height(1.2)
                        )
                        .padding(0)
                        .style(move |_| container::Style {
                            background: Some(bg_with_alpha.into()),
                            ..Default::default()
                        })
                    )
                    .padding([20, 10])
                )
                .width(Length::Fill)
                .height(Length::Fill)
            ]
        )
        .padding(2)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_with_alpha.into()),
            border: Border {
                color: self.theme.border,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}
