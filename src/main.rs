mod theme;
mod watcher;

use iced::widget::{container, column, text};
use iced::{Element, Event, Border, Color, Length, Task as Command, event};
use iced_layershell::actions::LayershellCustomActionWithId;
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use theme::{Theme, WalColors};
use watcher::ColorWatcher;

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

struct Launcher {
    theme: Theme,
    watcher: Option<ColorWatcher>,
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
            });

        let watcher = ColorWatcher::new().ok();

        (Self { theme, watcher }, Command::none())
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
                    match key {
                        keyboard::Key::Named(Named::Escape) => {
                            std::process::exit(0);
                        }
                        _ => {}
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
        let bg_with_alpha = Color::from_rgba(bg.r, bg.g, bg.b, 0.82);

        container(
            column![
                container(text(""))
                    .padding(9)
                    .height(Length::Fill)
                    .style(move |_| container::Style {
                        border: Border {
                            color: self.theme.accent,
                            width: 2.0,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    })
            ]
            .spacing(12)
        )
        .padding([11, 17])
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