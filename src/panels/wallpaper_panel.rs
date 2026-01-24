use iced::widget::{container, text, stack, image, row, button};
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
    selected: usize,
) -> Element<'a, Message> {
    let wallpaper_view: Element<'a, Message> = if let Some(index) = wallpapers {
        if let Some(entry) = index.wallpapers.get(selected) {
            let path = entry
                .thumbnail
                .as_ref()
                .unwrap_or(&entry.path);

            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(text("No wallpaper"))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        }
    } else {
        container(text("No wallpapers"))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };

    let controls = row![
        container(
            button(
                text("◀")
                    .font(font)
                    .size(font_size * 1.6)
                    .color(theme.color6)
            )
            .on_press(Message::PrevWallpaper)
        )
        .width(Length::FillPortion(1))
        .center_x(Length::Fill)
        .center_y(Length::Fill),

        container(text(""))
            .width(Length::FillPortion(3))
            .height(Length::Fill),

        container(
            button(
                text("▶")
                    .font(font)
                    .size(font_size * 1.6)
                    .color(theme.color6)
            )
            .on_press(Message::NextWallpaper)
        )
        .width(Length::FillPortion(1))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    ]
    .height(Length::Fill);

    let content = stack![
        container(wallpaper_view)
            .width(Length::Fill)
            .height(Length::Fill),
        container(controls)
            .width(Length::Fill)
            .height(Length::Fill)
    ];

    container(
        container(
            stack![
                container(
                    content
                )
                .padding(iced::padding::top(25))
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
