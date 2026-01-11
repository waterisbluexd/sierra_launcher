use iced::widget::{container, text};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;

pub fn right_main_panels_view<'a>(theme: &'a Theme, bg_with_alpha: Color) -> Element<'a, Message> {
    container(text("Right Main Panel Content")
        .width(Length::Fill)
        .height(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(bg_with_alpha.into()),
            border: Border {
                color: theme.border,
                width: 2.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
