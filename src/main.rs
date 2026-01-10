use iced::widget::{button, column, container, text};
use iced::{Element, Length, Task};
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};

pub fn main() -> Result<(), iced_layershell::Error> {
    iced_layershell::run::<SierraLauncher>(Settings {
        layer_settings: LayerShellSettings {
            size: Some((400, 300)),
            anchor: Anchor::Top | Anchor::Left,
            ..Default::default()
        },
        ..Default::default()
    })
}

struct SierraLauncher {
    counter: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}

impl iced::Application for SierraLauncher {
    type Message = Message;
    type Flags = ();
    type Theme = iced::Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (SierraLauncher { counter: 0 }, Task::none())
    }

    fn title(&self) -> String {
        String::from("Sierra Launcher")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.counter += 1,
            Message::Decrement => self.counter -= 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        container(
            column![
                text("Sierra Launcher").size(32),
                text(format!("Counter: {}", self.counter)),
                button("Increment").on_press(Message::Increment),
                button("Decrement").on_press(Message::Decrement),
            ]
            .spacing(20)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .into()
    }
}
