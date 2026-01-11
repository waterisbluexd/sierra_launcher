use iced::widget::{text_input, text_input::Id};
use iced::{Element, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    Submitted,
}

pub struct SearchBar {
pub input_value: String,
    input_id: text_input::Id,
}

impl SearchBar {
    pub fn new() -> Self {
        Self {
            input_value: String::new(),
            input_id: text_input::Id::unique(),
        }
    }

    pub fn view(&self, theme: &crate::utils::theme::Theme, font: iced::Font, font_size: f32) -> Element<Message> {
        text_input(
            "Search for apps...",
            &self.input_value,
        )
        .on_input(Message::InputChanged)
        .id(self.input_id.clone())
        .on_submit(Message::Submitted)
        .size(font_size)
        .font(font)
        .padding(10)
        .into()
    }
}
