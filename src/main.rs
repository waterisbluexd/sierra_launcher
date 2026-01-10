use iced::widget::{container,column,text};
use iced::{Element, Event, Border, Color, Length, Task as Command, event};
use iced_layershell::actions::LayershellCustomActionWithId;
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};

fn main() -> Result<(), iced_layershell::Error> {
    application(
        Launcher::new,
        Launcher::namespace,
        Launcher::update,
        Launcher::view,
    )
    .settings(Settings {
        layer_settings: LayerShellSettings {
            size: Some((484, 714)),
            anchor: Anchor::Bottom,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            margin: (0, 0, 4, 0),
            ..Default::default()
        },
        ..Default::default()
    })

    .style(|_theme, _id| iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    })
    .subscription(Launcher::subscription)
    .run()?;
    
    Ok(())
}

struct Launcher;

#[derive(Debug, Clone)]
enum Message {
    IcedEvent(Event),
}

impl TryInto<LayershellCustomActionWithId> for Message {
    type Error = Self;
    fn try_into(self) -> Result<LayershellCustomActionWithId, Self::Error> {
        Err(self)
    }
}

impl Launcher {
    fn new() -> (Self, Command<Message>) {
        (Self, Command::none())
    }

    fn namespace() -> String {
        String::from("iced_launcher2")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        event::listen().map(Message::IcedEvent)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        use iced::keyboard;
        use keyboard::key::Named;
        
        match message {
            Message::IcedEvent(event) => {
                if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                    match key {
                        keyboard::Key::Named(Named::Escape) => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
    container(
        column![
            text("")
                .color(Color::WHITE),

            // Container 2 (border only)
            container(text(""))
                .padding(10)
                .style(|_| container::Style {
                    border: Border {
                        color: Color::from_rgb(0.6, 0.6, 0.6),
                        width: 2.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
        ]
        .spacing(12)
    )
    .padding(15)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_| container::Style {
        background: Some(Color::from_rgba(0.1, 0.0, 0.0, 0.6).into()),
        border: Border {
            color: Color::from_rgb(0.5, 0.5, 0.5),
            width: 2.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
}