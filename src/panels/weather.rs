use iced::widget::{container, text, column, stack};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;
pub fn weather_panel_view<'a>(
theme: &'a Theme,
bg_with_alpha: Color,
font: iced::Font,
font_size: f32,
) -> Element<'a, Message> {
container(
        container(
        stack![
// Background + content container
            container(
                container(
                    column![
// ── CONTENT GOES HERE ──
                        text("here starts all the text
")
.color(theme.color6)
.font(font)
.size(font_size),
]
.padding(10)
)
.width(Length::Fill)
.height(Length::Fill)
.style(move |_| container::Style {
background: None,
border: Border {
color: theme.color3,
width: 2.0,
radius: 0.0.into(),
},
..Default::default()
})
)
.padding(iced::padding::top(15))
.width(Length::Fill)
.height(Length::Fill),
// Floating title label
            container(
                container(
                    text(" Weather ")
.color(theme.color6)
.font(font)
.size(font_size)
)
.width(Length::Shrink)
.height(Length::Shrink)
.style(move |_| container::Style {
background: Some(bg_with_alpha.into()),
..Default::default()
})
)
.padding(iced::padding::left(8).top(5))
.width(Length::Shrink)
.height(Length::Shrink),
]
)
.width(Length::Fill)
.height(Length::Fill)
.style(move |_| container::Style {
background: None,
..Default::default()
}),
    )
.width(Length::Fill)
.height(Length::FillPortion(1))
.style(move |_| container::Style {
background: None,
..Default::default()
})
.into()
}