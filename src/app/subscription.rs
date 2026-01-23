use iced::{Subscription, window, event};
use crate::app::message::Message;

pub fn subscription() -> Subscription<Message> {
    let events = event::listen().map(Message::IcedEvent);
    let frames = window::frames().map(|_| Message::CheckColors);
    let music_refresh = window::frames().map(|_| Message::MusicRefresh);

    Subscription::batch(vec![events, frames, music_refresh])
}
