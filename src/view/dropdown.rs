use crate::app::Notepad;
use crate::config::{
    DROPDOWN_BG, DROPDOWN_BORDER_COLOR, MENU_ITEM_HOVER_BG, PRETENDARD, SEPARATOR_COLOR,
};
use crate::message::{MenuState, Message};
use iced::widget::{button, column, container, text, Space};
use iced::{Border, Element, Length, Theme};

const MOD_KEY_N: &str = "New           Ctrl+N";
const MOD_KEY_O: &str = "Open...       Ctrl+O";
const MOD_KEY_S: &str = "Save          Ctrl+S";
const MOD_KEY_Z: &str = "Undo          Ctrl+Z";
const MOD_KEY_X: &str = "Cut           Ctrl+X";
const MOD_KEY_C: &str = "Copy          Ctrl+C";
const MOD_KEY_V: &str = "Paste         Ctrl+V";
const MOD_KEY_A: &str = "Select All    Ctrl+A";
const MOD_KEY_PLUS: &str = "Zoom In       Ctrl++";
const MOD_KEY_MINUS: &str = "Zoom Out      Ctrl+-";
const MOD_KEY_0: &str = "Reset Zoom    Ctrl+0";

fn disabled_menu_item(label: &'static str) -> Element<'static, Message> {
    container(text(label).font(PRETENDARD).style(|_: &Theme| text::Style {
        color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
    }))
    .padding([5.0, 10.0])
    .width(Length::Fill)
    .into()
}

pub fn view_dropdown(notepad: &Notepad) -> Element<'_, Message> {
    let dropdown = match notepad.menu_state {
        MenuState::File => view_file_dropdown(),
        MenuState::Edit => view_edit_dropdown(),
        MenuState::Format => view_format_dropdown(notepad),
        MenuState::View => view_view_dropdown(notepad),
        MenuState::Help => view_help_dropdown(),
        MenuState::None => return Space::new().into(),
    };

    let left_offset = match notepad.menu_state {
        MenuState::File => 5.0,
        MenuState::Edit => 55.0,
        MenuState::Format => 105.0,
        MenuState::View => 170.0,
        MenuState::Help => 230.0,
        MenuState::None => 0.0,
    };

    container(dropdown)
        .padding(iced::Padding::new(28.0).right(5.0).bottom(5.0).left(left_offset))
        .width(Length::Shrink)
        .height(Length::Shrink)
        .align_x(iced::Alignment::Start)
        .align_y(iced::Alignment::Start)
        .into()
}

pub fn view_file_dropdown() -> Element<'static, Message> {
    container(
        column![
            menu_item(MOD_KEY_N, Message::NewFile),
            menu_item(MOD_KEY_O, Message::OpenFile),
            menu_item(MOD_KEY_S, Message::SaveFile),
            menu_item("Save As...", Message::SaveFileAs),
            separator(),
            menu_item("Exit", Message::Exit),
        ]
        .spacing(2),
    )
    .padding(5)
    .style(|_: &Theme| container::Style {
        background: Some(DROPDOWN_BG.into()),
        border: Border {
            color: DROPDOWN_BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .width(180)
    .into()
}

pub fn view_edit_dropdown() -> Element<'static, Message> {
    container(
        column![
            disabled_menu_item(MOD_KEY_Z),
            separator(),
            menu_item(MOD_KEY_X, Message::Cut),
            menu_item(MOD_KEY_C, Message::Copy),
            menu_item(MOD_KEY_V, Message::Paste),
            menu_item("Delete        Del", Message::Delete),
            separator(),
            menu_item(MOD_KEY_A, Message::SelectAll),
            separator(),
            menu_item("Time/Date    F5", Message::TimeDate),
        ]
        .spacing(2),
    )
    .padding(5)
    .style(|_: &Theme| container::Style {
        background: Some(DROPDOWN_BG.into()),
        border: Border {
            color: DROPDOWN_BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .width(180)
    .into()
}

pub fn view_format_dropdown(notepad: &Notepad) -> Element<'_, Message> {
    let word_wrap_text = if notepad.word_wrap {
        "✓ Word Wrap"
    } else {
        "  Word Wrap"
    };

    container(
        column![menu_item(word_wrap_text, Message::WordWrapToggled)].spacing(2),
    )
    .padding(5)
    .style(|_: &Theme| container::Style {
        background: Some(DROPDOWN_BG.into()),
        border: Border {
            color: DROPDOWN_BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .width(150)
    .into()
}

pub fn view_view_dropdown(notepad: &Notepad) -> Element<'_, Message> {
    let status_bar_text = if notepad.show_status_bar {
        "✓ Status Bar"
    } else {
        "  Status Bar"
    };

    container(
        column![
            menu_item(MOD_KEY_PLUS, Message::ZoomIn),
            menu_item(MOD_KEY_MINUS, Message::ZoomOut),
            menu_item(MOD_KEY_0, Message::ZoomReset),
            separator(),
            menu_item(status_bar_text, Message::StatusBarToggled),
        ]
        .spacing(2),
    )
    .padding(5)
    .style(|_: &Theme| container::Style {
        background: Some(DROPDOWN_BG.into()),
        border: Border {
            color: DROPDOWN_BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .width(180)
    .into()
}

pub fn view_help_dropdown() -> Element<'static, Message> {
    container(
        column![menu_item("About MemoChan", Message::About)].spacing(2),
    )
    .padding(5)
    .style(|_: &Theme| container::Style {
        background: Some(DROPDOWN_BG.into()),
        border: Border {
            color: DROPDOWN_BORDER_COLOR,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .width(160)
    .into()
}

fn menu_item<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    button(text(label).font(PRETENDARD))
        .width(Length::Fill)
        .on_press(message)
        .style(|_: &Theme, status: button::Status| {
            let bg_color = match status {
                button::Status::Hovered => MENU_ITEM_HOVER_BG,
                _ => DROPDOWN_BG,
            };

            button::Style {
                background: Some(bg_color.into()),
                ..Default::default()
            }
        })
        .into()
}

fn separator() -> Element<'static, Message> {
    container(Space::new())
        .height(1.0)
        .style(|_: &Theme| container::Style {
            background: Some(SEPARATOR_COLOR.into()),
            ..Default::default()
        })
        .into()
}