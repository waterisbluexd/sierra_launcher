mod utils;
mod config;
mod panels;
use crate::panels::title_color::TitleAnimator;

use iced_layershell::application;
use iced::widget::{container, text, stack, row, column, operation::focus};
use iced::{Element, Event, Border, Color, Length, Task as Command, event};
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
    eprintln!("[Main] ========== STARTUP ==========");
    let app_start = std::time::Instant::now();
    eprintln!("[Main] Starting at: {:?}", app_start);
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
    Wallpaper,
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
    is_first_frame: bool,
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
    BluetoothToggle,
    EyeCareToggle,
    ToggleControlCenter,
    PowerOffTheSystem,
    RestartTheSystem,
    SleepModeTheSystem,
    ClipboardArrowUp,
    ClipboardArrowDown,
    ClipboardSelect,
    ClipboardDelete,
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

        let start = std::time::Instant::now();
    eprintln!("[Main] Starting initialization...");
    
    crate::utils::data::init();
    eprintln!("[Main] Clipboard init: {:?}", start.elapsed());
    
    let config = Config::load();
    eprintln!("[Main] Config load: {:?}", start.elapsed());
    let theme = Theme::load_from_config(&config);
    
        let _clipboard_monitor = crate::utils::monitor::start_monitor();
        
        // Load theme based on config preferences

        let watcher = ColorWatcher::new().ok();
        let search_bar = SearchBar::new();
        
        // ★ AppList::new() NOW RETURNS IMMEDIATELY - apps load in background ★
        let app_list = AppList::new();
        
        let weather_panel = WeatherPanel::new();
        let music_player = MusicPlayer::new();
        let system_panel = SystemPanel::new();
        let services_panel = ServicesPanel::new();

        let title_animator = TitleAnimator::new()
            .with_mode(config.get_animation_mode())
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
            is_first_frame: true,
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
                            
                            keyboard::Key::Named(Named::Enter) => {
                                if self.clipboard_visible {
                                    return Command::perform(async {}, |_| Message::ClipboardSelect);
                                } else {
                                    // Launch selected app
                                    let _ = self.app_list.update(app_list::Message::LaunchSelected);
                                    std::process::exit(0);
                                }
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
                                    self.clipboard_visible = !self.clipboard_visible;
                                } else {
                                    return Command::perform(async {}, |_| Message::CyclePanel(Direction::Left));
                                }
                            }
                            
                            keyboard::Key::Named(Named::ArrowRight) => {
                                if modifiers.shift() {
                                    self.clipboard_visible = !self.clipboard_visible;
                                } else {
                                    return Command::perform(async {}, |_| Message::CyclePanel(Direction::Right));
                                }
                            }

                            keyboard::Key::Named(Named::Backspace) => {
                                if !self.clipboard_visible && !self.search_bar.input_value.is_empty() {
                                    // Handle backspace for search input
                                    self.search_bar.input_value.pop();
                                    let _ = self.app_list.update(app_list::Message::SearchInput(self.search_bar.input_value.clone()));
                                }
                            }

                            keyboard::Key::Character(c) => {
                                if self.clipboard_visible && modifiers.control() && c.as_str() == "d" {
                                    return Command::perform(async {}, |_| Message::ClipboardDelete);
                                } else if !self.clipboard_visible && !modifiers.control() && !modifiers.alt() && !modifiers.logo() {
                                    // Type into search bar even when not focused
                                    self.search_bar.input_value.push_str(c.as_str());
                                    let _ = self.app_list.update(app_list::Message::SearchInput(self.search_bar.input_value.clone()));
                                }
                            }
                            
                            _ => {}
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                        self.control_center_visible = !self.control_center_visible;
                    }
                    _ => {}
                }

                Command::none()
            }

            Message::CheckColors => {
                // ★ FIRST FRAME: Focus search bar and trigger app loading ★
                if self.is_first_frame {
                    self.is_first_frame = false;
                    
                    // Trigger lazy loading of apps in background
                    self.app_list.start_loading();
                    eprintln!("[Main] Triggered lazy app loading");
                    self.system_panel.start();
                    return focus(self.search_bar.input_id.clone());
                }
                
                // ★ CHECK IF APPS FINISHED LOADING ★
                if self.app_list.check_loaded() {
                    eprintln!("[Main] Apps finished loading - UI will update automatically");
                }
                
                self.frame_count += 1;
                
                self.title_animator.update();
                
                let now = Instant::now();
                if now.duration_since(self.last_color_check) > Duration::from_secs(1) {
                    self.last_color_check = now;
                    
                    // Only check for pywal changes if pywal is enabled
                    if self.config.use_pywal {
                        if let Some(ref watcher) = self.watcher {
                            if watcher.check_for_changes() {
                                if let Ok(wal_colors) = WalColors::load() {
                                    self.theme = wal_colors.to_theme();
                                    eprintln!("Pywal theme reloaded");
                                }
                            }
                        }
                    }
                }
                
                if now.duration_since(self.last_services_refresh) > Duration::from_secs(5) {
                    self.last_services_refresh = now;
                    
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
                        let _ = self.app_list.update(app_list::Message::LaunchSelected);
                        std::process::exit(0);
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
                    (Panel::Music, Direction::Right) => Panel::Wallpaper,
                    (Panel::Wallpaper, Direction::Right) => Panel::System,
                    (Panel::System, Direction::Right) => Panel::Services,
                    (Panel::Services, Direction::Right) => Panel::Clock,
                    (Panel::Clock, Direction::Left) => Panel::Services,
                    (Panel::Services, Direction::Left) => Panel::System,
                    (Panel::System, Direction::Left) => Panel::Wallpaper,
                    (Panel::Wallpaper, Direction::Left) => Panel::Music,
                    (Panel::Music, Direction::Left) => Panel::Weather,
                    (Panel::Weather, Direction::Left) => Panel::Clock,
                };
                
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
                self.control_center_visible = false;
                
                std::thread::spawn(|| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("poweroff")
                        .output();
                });
                Command::none()
            }

            Message::RestartTheSystem => {
                self.control_center_visible = false;
                
                std::thread::spawn(|| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("reboot")
                        .output();
                });
                Command::none()
            }

            Message::SleepModeTheSystem => {
                self.control_center_visible = false;
                
                let _ = std::process::Command::new("bash")
                    .arg("-c")
                    .arg("(sleep 0.5 && systemctl suspend) &")
                    .spawn();
                
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
                        let content = item.full_content();
                    
                        crate::utils::monitor::set_ignore_next(content.clone());
                        let _ = crate::utils::copy::copy_to_clipboard(&content);
                    }
                    Command::none()
                }


            Message::ClipboardDelete => {
                let items = crate::utils::data::search_items("");
                if !items.is_empty() && self.clipboard_selected_index < items.len() {
                    crate::utils::data::delete_item(self.clipboard_selected_index);
                    
                    let new_count = crate::utils::data::item_count();
                    if self.clipboard_selected_index >= new_count && new_count > 0 {
                        self.clipboard_selected_index = new_count - 1;
                    } else if new_count == 0 {
                        self.clipboard_selected_index = 0;
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

        // Get Font from config using helper method
        let font = self.config.get_font();
        let font_size = self.config.font_size.unwrap_or(22.0);
        
        let title_text = &self.config.title_text;
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