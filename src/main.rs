use iced::widget::container;
use iced::{Element, Event, Length, Task as Command, event};
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
        container("")
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.42, // R
                    0.31, // G
                    0.22, // B
                ))),
                ..Default::default()
            })
            .into()
    }
}