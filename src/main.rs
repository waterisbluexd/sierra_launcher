mod utils;
mod config;
mod panels;
use crate::panels::title_color::{TitleAnimator, AnimationMode};

use iced_layershell::application;
use iced::widget::{container, text, stack, row, column};
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

use std::time::{Duration, Instant};

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
    last_color_check: Instant,
    last_services_refresh: Instant,
    frame_count: u32,
    title_animator: TitleAnimator,
    control_center_visible: bool,
    clipboard_visible: bool,
    clipboard_selected_index: usize,
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
    VolumeMuteToggle,
    AirplaneModeToggle,
    BrightnessMinToggle,
    WifiToggle,
    WifiRefresh,
    BluetoothToggle,
    EyeCareToggle,
    ToggleControlCenter,
    PowerOffTheSystem,
    RestartTheSystem,
    SleepModeTheSystem,
    ClipboardArrowUp,
    ClipboardArrowDown,
    ClipboardSelect,
    NoOp,
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
        crate::utils::data::init();
        let _clipboard_monitor = crate::utils::monitor::start_monitor();
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

        // AnimationMode::Rainbow
        // AnimationMode::Wave
        // AnimationMode::InOutWave
        // AnimationMode::Pulse
        // AnimationMode::Sparkle
        // AnimationMode::Gradient
        let title_animator = TitleAnimator::new()
            .with_mode(AnimationMode::Wave)
            .with_speed(80);

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
            last_color_check: Instant::now(),
            last_services_refresh: Instant::now(),
            frame_count: 0,
            title_animator,
            control_center_visible: false,
            clipboard_visible: false,
            clipboard_selected_index: 0,
        }, Command::none())
    }

    fn namespace() -> String {
        String::from("iced_launcher2")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::window;

        let events = event::listen().map(Message::IcedEvent);
        let frames = window::frames().map(|_| Message::CheckColors);
        let music_refresh = window::frames().map(|_| Message::MusicRefresh);

        iced::Subscription::batch(vec![events, frames, music_refresh])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        use iced::keyboard;
        use keyboard::key::Named;
        use iced::mouse;

        match message {
            Message::IcedEvent(event) => {
                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                        match key {
                            keyboard::Key::Named(Named::Escape) => {
                                std::process::exit(0);
                            }
                            keyboard::Key::Named(Named::ArrowUp) => {
                                if self.clipboard_visible {
                                    return Command::perform(async {}, |_| Message::ClipboardArrowUp);
                                } else {
                                    let _ = self.app_list.update(app_list::Message::ArrowUp);
                                }
                            }
                            keyboard::Key::Named(Named::ArrowDown) => {
                                if self.clipboard_visible {
                                    return Command::perform(async {}, |_| Message::ClipboardArrowDown);
                                } else {
                                    let _ = self.app_list.update(app_list::Message::ArrowDown);
                                }
                            }
                            keyboard::Key::Named(Named::ArrowLeft) => {
                                if modifiers.shift() {
                                    // Shift+Left: Toggle clipboard panel
                                    self.clipboard_visible = !self.clipboard_visible;
                                } else {
                                    return Command::perform(async {}, |_| Message::CyclePanel(Direction::Left));
                                }
                            }
                            keyboard::Key::Named(Named::ArrowRight) => {
                                if modifiers.shift() {
                                    // Shift+Right: Toggle clipboard panel
                                    self.clipboard_visible = !self.clipboard_visible;
                                } else {
                                    return Command::perform(async {}, |_| Message::CyclePanel(Direction::Right));
                                }
                            }
                            keyboard::Key::Named(Named::Enter) => {
                                if self.clipboard_visible {
                                    return Command::perform(async {}, |_| Message::ClipboardSelect);
                                } else {
                                    let _ = self.app_list.update(app_list::Message::LaunchSelected);
                                }
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                        // Right-click to toggle control center
                        self.control_center_visible = !self.control_center_visible;
                    }
                    _ => {}
                }

                Command::none()
            }

            Message::CheckColors => {
                self.frame_count += 1;
                
                // Update title animation
                self.title_animator.update();
                
                // Only check colors every 60 frames (~1 second at 60fps)
                let now = Instant::now();
                if now.duration_since(self.last_color_check) > Duration::from_secs(1) {
                    self.last_color_check = now;
                    
                    if let Some(ref watcher) = self.watcher {
                        if watcher.check_for_changes() {
                            if let Ok(wal_colors) = WalColors::load() {
                                self.theme = wal_colors.to_theme();
                            }
                        }
                    }
                }
                
                // Refresh WiFi and Bluetooth status every 5 seconds only
                if now.duration_since(self.last_services_refresh) > Duration::from_secs(5) {
                    self.last_services_refresh = now;
                    
                    // Only refresh if Services panel is active
                    if self.current_panel == Panel::Services {
                        self.services_panel.schedule_refresh();
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
                
                // Trigger immediate refresh when switching to Services panel
                if self.current_panel == Panel::Services {
                    self.services_panel.schedule_refresh();
                }
                
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

            Message::VolumeMuteToggle => {
                self.services_panel.toggle_mute();
                Command::none()
            }
            
            Message::AirplaneModeToggle => {
                self.services_panel.toggle_airplane_mode();
                Command::none()
            }
            
            Message::BrightnessMinToggle => {
                self.services_panel.toggle_min_brightness();
                Command::none()
            }

            Message::WifiToggle => {
                self.services_panel.toggle_wifi();
                Command::none()
            }

            Message::WifiRefresh => {
                self.services_panel.schedule_refresh();
                Command::none()
            }

            Message::BluetoothToggle => {
                self.services_panel.toggle_bluetooth();
                Command::none()
            }

            Message::EyeCareToggle => {
                self.services_panel.toggle_eye_care();
                Command::none()
            }

            Message::ToggleControlCenter => {
                self.control_center_visible = !self.control_center_visible;
                Command::none()
            }

            Message::PowerOffTheSystem => {
                // Hide control center when action is taken
                self.control_center_visible = false;
                
                // Use systemctl which works without sudo on most modern systems
                std::thread::spawn(|| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("poweroff")
                        .output();
                });
                Command::none()
            }

            Message::RestartTheSystem => {
                // Hide control center when action is taken
                self.control_center_visible = false;
                
                std::thread::spawn(|| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("reboot")
                        .output();
                });
                Command::none()
            }

            Message::SleepModeTheSystem => {
                // Hide control center
                self.control_center_visible = false;
                
                // Approach: Fork a shell command that will outlive the process
                let _ = std::process::Command::new("bash")
                    .arg("-c")
                    .arg("(sleep 0.5 && systemctl suspend) &")
                    .spawn();
                
                // Exit the launcher immediately
                std::process::exit(0);
            }

            Message::ClipboardArrowUp => {
                if self.clipboard_selected_index > 0 {
                    self.clipboard_selected_index -= 1;
                }
                Command::none()
            }

            Message::ClipboardArrowDown => {
                let items = crate::utils::data::search_items("");
                if self.clipboard_selected_index + 1 < items.len() {
                    self.clipboard_selected_index += 1;
                }
                Command::none()
            }

            Message::ClipboardSelect => {
                let items = crate::utils::data::search_items("");
                if let Some(item) = items.get(self.clipboard_selected_index) {
                    // Copy selected item to clipboard
                    use arboard::Clipboard;
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let content = item.full_content();
                        let _ = clipboard.set_text(content);
                    }
                }
                Command::none()
            }

            Message::NoOp => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let bg = self.theme.background;
        let bg_with_alpha = Color::from_rgb(bg.r, bg.g, bg.b);

        let font = match self.config.font.as_deref() {
            Some("Monocraft") => Font::with_name("Monocraft"),
            Some("Monospace") => Font::with_name("Monospace"),
            _ => Font::default(),
        };

        let font_size = self.config.font_size.unwrap_or(22.0);
        
        // Create animated vertical text with individual character colors
        let title_text = " sierra-launcher ";
        let total_chars = title_text.chars().count();
        let mut title_column = column![].spacing(0);
        
        for (i, ch) in title_text.chars().enumerate() {
            let char_color = self.title_animator.get_color_for_char(&self.theme, i, total_chars);
            title_column = title_column.push(
                text(ch.to_string())
                    .font(font)
                    .size(font_size)
                    .color(char_color)
            );
        }

        container(
            stack![
                container(
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
                container(
                    row![
                        container(text(""))
                            .height(Length::Fill)
                            .width(Length::Shrink),
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
                            self.control_center_visible,
                            self.clipboard_visible,
                            self.clipboard_selected_index,
                        ))
                        .height(Length::Fill)
                        .width(Length::Fill),
                    ]
                    .spacing(45)
                )
                .padding(iced::padding::bottom(14).right(14))
                .width(Length::Fill)
                .height(Length::Fill),
                container(
                    container(
                        container(title_column)
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