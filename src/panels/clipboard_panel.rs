use iced::widget::{container, text, stack, column, scrollable};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;

pub fn clipboard_panel_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    // Get clipboard history items
    let items = crate::utils::data::search_items("");
    
    // Build list similar to app_list
    let mut list = column![].spacing(1);
    
    if items.is_empty() {
        list = list.push(
            container(
                text("No clipboard history yet")
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .padding(10)
            .width(Length::Fill)
        );
    } else {
        for (idx, item) in items.iter().enumerate() {
            let preview = item.preview();
            let bg = if idx % 2 == 0 {
                Some(Color::from_rgba(theme.color0.r, theme.color0.g, theme.color0.b, 0.1).into())
            } else {
                None
            };
            
            list = list.push(
                container(
                    text(preview)
                        .font(font)
                        .size(font_size * 0.8)
                        .color(theme.foreground)
                )
                .padding([4, 8])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: bg,
                    border: Border::default(),
                    ..Default::default()
                })
            );
        }
    }
    
    container(
        stack![
            container(
                container(
                    scrollable(list)
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::padding::top(15).right(15).left(15))
                .style(move |_| container::Style {
                    background: None,
                    ..Default::default()
                }),
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: None,
                border: Border {
                    color: theme.color3,
                    width: 2.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }),
            
            container(
                text(" Clipboard ")
                    .color(theme.color6)
                    .font(font)
                    .size(font_size)
            )
            .padding(iced::padding::top(9))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: None,
                ..Default::default()
            }),

            container(
                container(
                    text(" Clipboard ")
                        .color(theme.color6)
                        .font(font)
                        .size(font_size)
                )
                .width(Length::Shrink)
                .height(Length::Shrink)
                .style(move |_| container::Style {
                    background: Some(bg_with_alpha.into()),
                    ..Default::default()
                }),
            )
            .padding(iced::padding::left(8))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .style(move |_| container::Style {
                background: None,
                ..Default::default()
            }),
        ]
    )
    .padding(iced::padding::top(219))
    .width(Length::Fill)
    .height(Length::FillPortion(1))
    .style(move |_| container::Style {
        background: None,
        ..Default::default()
    })
    .into()
}