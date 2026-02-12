mod app;
mod config;
mod message;
mod view;

use crate::app::Notepad;
use crate::config::PRETENDARD_FONT;

fn main() -> iced::Result {
    iced::application(Notepad::new, Notepad::update, Notepad::view)
        .font(PRETENDARD_FONT)
        .window_size(iced::Size::new(800.0, 600.0))
        .theme(Notepad::theme)
        .title(Notepad::title)
        .subscription(Notepad::subscription)
        .run()
}
