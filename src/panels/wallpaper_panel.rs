use iced::widget::{container, text, stack, image, column};
use iced::{Element, Color, Length, Border};

use crate::utils::theme::Theme;
use crate::utils::wallpaper_manager::WallpaperIndex;
use crate::Message;

pub fn wallpaper_panel_view<'a>(
    theme: &'a Theme,
    bg_with_alpha: Color,
    font: iced::Font,
    font_size: f32,
    wallpapers: Option<&'a WallpaperIndex>,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = if let Some(index) = wallpapers {
        let items = index.wallpapers.iter().take(12).map(|entry| {
            if let Some(thumbnail) = &entry.thumbnail {
                image(image::Handle::from_path(thumbnail))
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(80.0))
                    .into()
            } else {
                container(
                    text(&entry.name)
                        .color(theme.color6)
                        .size(12),
                )
                .width(Length::Fixed(120.0))
                .height(Length::Fixed(80.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            }
        });

        column(items)
            .spacing(10)
            .padding(10)
            .into()
    } else {
        container(text("No wallpapers found"))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };
    container(
        container(
            stack![
                container(container(
                    container(content)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::padding::top(25))
                .style(move |_| container::Style {
                    background: None,
                    border: Border {
                        color: theme.color3,
                        width: 2.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(Length::Fill),)
                .padding(iced::padding::top(15))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| container::Style {
                    background: None,
                    ..Default::default()
                }),
                

                container(
                    container(
                        text(" Wallpapers ")
                            .color(theme.color6)
                            .font(font)
                            .size(font_size),
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
    )
    .width(Length::Fill)
    .height(Length::FillPortion(1))
    .into()
}
