use iced::widget::{column, container, row, scrollable, text};
use iced::{Border, Element, Length};
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
}

impl AppList {
    pub fn new() -> Self {
        let all_apps = Self::load_desktop_apps();

        Self {
            filtered_apps: all_apps.clone(),
            all_apps,
            search_query: String::new(),
            selected_index: 0,
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
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SearchInput(query) => {
                self.search_query = query;
                self.filter_apps();
            }
            Message::ArrowUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Message::ArrowDown => {
                if self.selected_index + 1 < self.filtered_apps.len() {
                    self.selected_index += 1;
                }
            }
            Message::LaunchSelected => {
                self.launch_selected();
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

        for (idx, app) in self.filtered_apps.iter().enumerate() {
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