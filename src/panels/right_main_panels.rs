use iced::widget::{container, text, column};
use iced::{Element, Border, Color, Length};
use crate::utils::theme::Theme;
use crate::Message;

pub fn right_main_panels_view<'a>(theme: &'a Theme, bg_with_alpha: Color) -> Element<'a, Message> {
    container(
        column![
            // Container 1 - with min/max height constraints
            container(text("Panel 1"))
                .width(Length::Fill)
                .height(Length::FillPortion(1))  // Takes 1 portion of available space
                .style(move |_| container::Style {
                    background: Some(Color::from_rgb(0.2, 0.3, 0.4).into()),
                    border: Border {
                        color: theme.border,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }),
            
            // Container 2 - with different sizing
            container(text("Panel 2"))
                .width(Length::Fill)
                .height(Length::FillPortion(2))  // Takes 2 portions (twice as tall as panel 1)
                .style(move |_| container::Style {
                    background: Some(Color::from_rgb(0.3, 0.4, 0.5).into()),
                    border: Border {
                        color: theme.border,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }),
            
            // Container 3 - with fixed height
            container(text("Panel 3"))
                .width(Length::Fill)
                .height(Length::Fixed(150.0))  // Fixed height of 150 pixels
                .style(move |_| container::Style {
                    background: Some(Color::from_rgb(0.4, 0.5, 0.6).into()),
                    border: Border {
                        color: theme.border,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
        ]
        .spacing(10)  // Add spacing between containers
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