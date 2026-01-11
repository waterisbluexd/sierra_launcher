use iced::widget::{column, text};
use iced::{Element, Length};
use crate::utils::theme::Theme;

// Define a message enum for AppList, if it needs to handle its own events.
// For now, it will primarily receive updates from the search bar.
#[derive(Debug, Clone)]
pub enum Message {
    SearchInput(String),
    // Add other messages if AppList needs to handle user interactions itself,
    // e.g., selecting an app, scrolling, etc.
}

pub struct AppList {
    // This will eventually hold the list of applications,
    // and potentially filtered applications based on search input.
    pub search_query: String, // Store the current search query
}

impl AppList {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SearchInput(query) => {
                self.search_query = query;
                // Here is where you would typically filter your app list
                // based on the new search query.
                // For now, we just update the internal state.
                println!("AppList received search query: {}", self.search_query);
            }
        }
    }

    pub fn view<'a>(&self, theme: &'a Theme, font: iced::Font, font_size: f32) -> Element<'a, Message> {
        column![
            text(format!("Search Query: {}", self.search_query))
                .font(font)
                .size(font_size)
                .color(theme.foreground),
            text("App List will go here...")
                .font(font)
                .size(font_size)
                .color(theme.foreground),
            // This is where the actual list of apps would be rendered
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
