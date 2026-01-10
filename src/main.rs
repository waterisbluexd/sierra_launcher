use iced::widget::container;
use iced::{Element, Event, Border,Background, Color, Length, Task as Command, event};
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
            size: Some((480, 710)),
            anchor: Anchor::Bottom,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            margin: (0, 0, 10, 0), //TRBL
            ..Default::default()
        },
        ..Default::default()
    })
    .subscription(Launcher::subscription)
    .run()?;
    std::thread::sleep(std::time::Duration::from_millis(1));
    Ok(())
}

impl TryInto<LayershellCustomActionWithId> for Message {
    type Error = Self;
    fn try_into(self) -> Result<LayershellCustomActionWithId, Self::Error> {
        Err(self)
    }
}

struct Launcher;

#[derive(Debug, Clone)]
enum Message {
    IcedEvent(Event),
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
                            return Command::done(std::process::exit(0));
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
            container("")
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.9))), 
                    border: Border {
                        color: Color::from_rgb(0.4, 0.4, 0.4), 
                        width: 1.0,
                        radius: 0.0.into(), 
                    },
                    ..Default::default()
                })
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: None,
            ..Default::default()
        })
        .into()
    }

}