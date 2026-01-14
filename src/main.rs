mod utils;
mod config;
mod panels;

use iced_layershell::application;
use iced::widget::{container, text, stack, row};
use iced::{Element, Event, Border, Color, Length, Font, Task as Command, event};
use iced_layershell::actions::LayershellCustomActionWithId;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use crate::utils::theme::{Theme, WalColors};
use crate::utils::watcher::ColorWatcher;
use crate::config::Config;
use crate::panels::search_bar::{self, SearchBar};
use crate::panels::app_list::{self, AppList};
use crate::panels::right_main_panels::right_main_panels_view;
use crate::panels::mpris_player::MusicPlayer;
use crate::panels::system::SystemPanel;
use crate::panels::services::ServicesPanel;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Clock,
    Weather,
    Music,
    System,
    Services,
}

use crate::panels::weather::WeatherPanel;

struct Launcher {
    theme: Theme,
    watcher: Option<ColorWatcher>,
    config: Config,
    search_bar: SearchBar,
    app_list: AppList,
    current_panel: Panel,
    weather_panel: WeatherPanel,
    music_player: MusicPlayer,
    system_panel: SystemPanel,
    services_panel: ServicesPanel,
}

#[derive(Debug, Clone)]
enum Message {
    IcedEvent(Event),
    CheckColors,
    SearchBarMessage(search_bar::Message),
    AppListMessage(app_list::Message),
    CyclePanel(Direction),
    MusicPlayPause,
    MusicNext,
    MusicPrevious,
    MusicProgressChanged(f32),
    MusicRefresh,
    VolumeChanged(f32),
    BrightnessChanged(f32),
    AudioMuteToggle,
    BrightnessMinToggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
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
        let search_bar = SearchBar::new();
        let app_list = AppList::new();
        let weather_panel = WeatherPanel::new();
        let music_player = MusicPlayer::new();
        let system_panel = SystemPanel::new();
        let services_panel = ServicesPanel::new();

        (Self { 
            theme, 
            watcher, 
            config, 
            search_bar, 
            app_list, 
            current_panel: Panel::Clock,
            weather_panel,
            music_player,
            system_panel,
            services_panel,
        }, Command::none())
    }

    fn namespace() -> String {
        String::from("iced_launcher2")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::window;

        let events = event::listen().map(Message::IcedEvent);
        let frames = window::frames().map(|_| Message::CheckColors);
        // Use frames for music refresh too - it updates frequently enough
        let music_refresh = window::frames().map(|_| Message::MusicRefresh);

        iced::Subscription::batch(vec![events, frames, music_refresh])
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
                        keyboard::Key::Named(Named::ArrowUp) => {
                            let _ = self.app_list.update(app_list::Message::ArrowUp);
                        }
                        keyboard::Key::Named(Named::ArrowDown) => {
                            let _ = self.app_list.update(app_list::Message::ArrowDown);
                        }
                        keyboard::Key::Named(Named::ArrowLeft) => {
                            return Command::perform(async {}, |_| Message::CyclePanel(Direction::Left));
                        }
                        keyboard::Key::Named(Named::ArrowRight) => {
                            return Command::perform(async {}, |_| Message::CyclePanel(Direction::Right));
                        }
                        keyboard::Key::Named(Named::Enter) => {
                            let _ = self.app_list.update(app_list::Message::LaunchSelected);
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
            
            Message::SearchBarMessage(search_bar_message) => {
                match search_bar_message {
                    search_bar::Message::InputChanged(value) => {
                        self.search_bar.input_value = value.clone();
                        let _ = self.app_list.update(app_list::Message::SearchInput(value));
                        Command::none()
                    }
                    search_bar::Message::Submitted => {
                        println!("Search submitted: {}", self.search_bar.input_value);
                        Command::none()
                    }
                }
            }
            
            Message::AppListMessage(app_list_message) => {
                let _ = self.app_list.update(app_list_message);
                Command::none()
            }

            Message::CyclePanel(direction) => {
                self.current_panel = match (self.current_panel, direction) {
                    (Panel::Clock, Direction::Right) => Panel::Weather,
                    (Panel::Weather, Direction::Right) => Panel::Music,
                    (Panel::Music, Direction::Right) => Panel::System,
                    (Panel::System, Direction::Right) => Panel::Services,
                    (Panel::Services, Direction::Right) => Panel::Clock,
                    (Panel::Clock, Direction::Left) => Panel::Services,
                    (Panel::Services, Direction::Left) => Panel::System,
                    (Panel::System, Direction::Left) => Panel::Music,
                    (Panel::Music, Direction::Left) => Panel::Weather,
                    (Panel::Weather, Direction::Left) => Panel::Clock,
                };
                Command::none()
            }

            Message::MusicPlayPause => {
                self.music_player.play_pause();
                Command::none()
            }

            Message::MusicNext => {
                self.music_player.next_track();
                Command::none()
            }

            Message::MusicPrevious => {
                self.music_player.previous_track();
                Command::none()
            }

            Message::MusicProgressChanged(position) => {
                self.music_player.seek_to(position);
                Command::none()
            }

            Message::MusicRefresh => {
                self.music_player.refresh_player();
                Command::none()
            }

            Message::VolumeChanged(value) => {
                self.services_panel.set_volume(value);
                Command::none()
            }

            Message::BrightnessChanged(value) => {
                self.services_panel.set_brightness(value);
                Command::none()
            }
            Message::AudioMuteToggle => {
                self.services_panel.toggle_mute();
                Command::none()
            }
            Message::BrightnessMinToggle => {
                self.services_panel.toggle_min_brightness();
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
                    // Container 4 (left)
                    container(text(""))
                        .padding(9)
                        .height(Length::Fill)
                        .width(Length::Shrink)
                        .style(move |_| container::Style {
                            background: Some(bg_with_alpha.into()),
                            border: Border {
                                color: self.theme.color6,
                                width: 2.0,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        }),
                )
                .padding(14)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(bg_with_alpha.into()),
                    ..Default::default()
                }),
                // =========================
                // Container 1 (right)
                // =========================
                container(
                    row![
                        // First container: height = Fill, width = Shrink
                        container(text(""))
                            .height(Length::Fill)
                            .width(Length::Shrink),
                        // Second container: height = Fill, width = Fill
                        container(right_main_panels_view(
                            &self.theme,
                            bg_with_alpha,
                            font,
                            font_size,
                            &self.search_bar,
                            &self.app_list,
                            self.current_panel,
                            &self.weather_panel,
                            &self.music_player,
                            &self.system_panel,
                            &self.services_panel,
                        ))
                        .height(Length::Fill)
                        .width(Length::Fill),
                    ]
                    .spacing(45)
                )
                .padding(iced::padding::bottom(14).right(14))
                .width(Length::Fill)
                .height(Length::Fill),
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