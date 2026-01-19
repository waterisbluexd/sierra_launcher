use iced::widget::{column, container, row, scrollable, text};
use iced::{Border, Element, Length, Task};
use gio::prelude::*;
use gio::{AppLaunchContext, DesktopAppInfo};

use crate::utils::theme::Theme;

#[derive(Debug, Clone)]
pub enum Message {
    SearchInput(String),
    ArrowUp,
    ArrowDown,
    LaunchSelected,
}

#[derive(Debug, Clone)]
pub struct App {
    appinfo: DesktopAppInfo,
    name: String,
    description: Option<String>,
}

pub struct AppList {
    all_apps: Vec<App>,
    filtered_apps: Vec<App>,
    pub search_query: String,
    pub selected_index: usize,
    scroll_id: iced::widget::Id,
    
    // Virtual scrolling parameters
    window_size: usize,  // Number of items to display at once (e.g., 10)
    window_start: usize, // Starting index of the visible window
}

impl AppList {
    pub fn new() -> Self {
        let all_apps = Self::load_desktop_apps();

        Self {
            filtered_apps: all_apps.clone(),
            all_apps,
            search_query: String::new(),
            selected_index: 0,
            scroll_id: iced::widget::Id::unique(),
            window_size: 17,
            window_start: 0,
        }
    }

    fn load_desktop_apps() -> Vec<App> {
        let mut apps = Vec::new();

        for app in gio::AppInfo::all() {
            let Ok(desktop) = app.downcast::<DesktopAppInfo>() else {
                continue;
            };

            if !desktop.should_show() {
                continue;
            }

            apps.push(App {
                name: desktop.name().to_string(),
                description: desktop.description().map(|d| d.to_string()),
                appinfo: desktop,
            });
        }

        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        apps
    }

    fn filter_apps(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_apps = self.all_apps.clone();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_apps = self
                .all_apps
                .iter()
                .filter(|app| {
                    app.name.to_lowercase().contains(&q)
                        || app
                            .description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&q))
                            .unwrap_or(false)
                })
                .cloned()
                .collect();
        }

        if self.selected_index >= self.filtered_apps.len() {
            self.selected_index = 0;
        }
        
        // Reset window when filtering changes
        self.update_window();
    }

    fn update_window(&mut self) {
        if self.filtered_apps.is_empty() {
            self.window_start = 0;
            return;
        }

        // Ensure selected item is within the visible window
        // When selection reaches the last 2 items of window, slide window down
        if self.selected_index >= self.window_start + self.window_size - 1 {
            self.window_start = (self.selected_index + 1).saturating_sub(self.window_size);
        }
        // When selection reaches the first 2 items of window, slide window up
        else if self.selected_index < self.window_start + 1 {
            self.window_start = self.selected_index.saturating_sub(1);
        }

        // Ensure window doesn't go past the end
        let max_start = self.filtered_apps.len().saturating_sub(self.window_size);
        if self.window_start > max_start {
            self.window_start = max_start;
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchInput(query) => {
                self.search_query = query;
                self.filter_apps();
                Task::none()
            }
            Message::ArrowUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.update_window();
                }
                Task::none()
            }
            Message::ArrowDown => {
                if self.selected_index + 1 < self.filtered_apps.len() {
                    self.selected_index += 1;
                    self.update_window();
                }
                Task::none()
            }
            Message::LaunchSelected => {
                self.launch_selected();
                Task::none()
            }
        }
    }

    fn launch_selected(&self) {
        if let Some(app) = self.filtered_apps.get(self.selected_index) {
            let _ = app.appinfo.launch(&[], AppLaunchContext::NONE);
        }
    }

    pub fn view<'a>(
        &'a self,
        theme: &'a Theme,
        font: iced::Font,
        font_size: f32,
    ) -> Element<'a, Message> {
        let mut items = column![].spacing(1);

        // Calculate the visible range
        let window_end = (self.window_start + self.window_size).min(self.filtered_apps.len());
        
        // Only render items within the window
        for idx in self.window_start..window_end {
            let app = &self.filtered_apps[idx];
            let selected = idx == self.selected_index;

            let bg = if selected {
                Some(theme.color3.into())
            } else {
                None
            };

            let fg = if selected {
                theme.background
            } else {
                theme.foreground
            };

            let content = if selected {
                row![
                    text(">>").font(font).size(font_size).color(fg),
                    text(&app.name).font(font).size(font_size).color(fg),
                ]
                .spacing(4)
            } else {
                row![
                    text("  ").font(font).size(font_size).color(fg),
                    text(&app.name).font(font).size(font_size).color(fg),
                ]
                .spacing(4)
            };

            items = items.push(
                container(content)
                    .padding([2, 4])
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: bg,
                        border: Border::default(),
                        ..Default::default()
                    }),
            );
        }
        scrollable(items)
            .id(self.scroll_id.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_, _| scrollable::Style {
                container: container::Style::default(),
                vertical_rail: scrollable::Rail {
                    background: None,
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        background: iced::Background::Color(iced::Color::TRANSPARENT),
                        border: Border::default(),
                    },
                },
                horizontal_rail: scrollable::Rail {
                    background: None,
                    border: Border::default(),
                    scroller: scrollable::Scroller {
                        background: iced::Background::Color(iced::Color::TRANSPARENT),
                        border: Border::default(),
                    },
                },
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: iced::Background::Color(iced::Color::TRANSPARENT),
                    border: Border::default(),
                    icon: iced::Color::TRANSPARENT,
                    shadow: iced::Shadow::default(),
                },
            })
            .into()
    }
}