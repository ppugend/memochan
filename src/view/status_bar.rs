use crate::app::Notepad;
use crate::config::{PRETENDARD, STATUS_BAR_BG};
use crate::message::Message;
use iced::widget::{container, row, text, Space};
use iced::{Alignment, Element, Length, Theme};

pub fn view_status_bar(notepad: &Notepad) -> Element<'_, Message> {
    let (line, column) = notepad.get_cursor_position();

    let status_text = format!("Ln: {}, Col: {}", line, column);

    container(
        row![
            text(status_text).font(PRETENDARD).size(12),
            container(Space::new()).width(Length::Fill),
            text(format!("{}%", notepad.zoom_level))
                .font(PRETENDARD)
                .size(12),
            container(Space::new()).width(10.0),
            text("UTF-8").font(PRETENDARD).size(12),
        ]
        .align_y(Alignment::Center)
        .padding([4.0, 10.0]),
    )
    .width(Length::Fill)
    .height(Length::Shrink)
    .style(|_: &Theme| container::Style {
        background: Some(STATUS_BAR_BG.into()),
        ..Default::default()
    })
    .into()
}