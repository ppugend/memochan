use crate::app::Notepad;
use crate::config::{MENU_BAR_BG, MENU_BUTTON_HOVER_BG, PRETENDARD};
use crate::message::{MenuState, Message};
use iced::widget::{button, container, mouse_area, row, text};
use iced::{Element, Length, Theme};

pub fn view_menu_bar(notepad: &Notepad) -> Element<'_, Message> {
    let file_button = view_menu_button(
        "File",
        MenuState::File,
        Message::FileMenuToggled,
        &notepad.menu_state,
    );
    let edit_button = view_menu_button(
        "Edit",
        MenuState::Edit,
        Message::EditMenuToggled,
        &notepad.menu_state,
    );
    let format_button = view_menu_button(
        "Format",
        MenuState::Format,
        Message::FormatMenuToggled,
        &notepad.menu_state,
    );
    let view_button = view_menu_button(
        "View",
        MenuState::View,
        Message::ViewMenuToggled,
        &notepad.menu_state,
    );
    let help_button = view_menu_button(
        "Help",
        MenuState::Help,
        Message::HelpMenuToggled,
        &notepad.menu_state,
    );

    container(
        row![
            file_button,
            edit_button,
            format_button,
            view_button,
            help_button,
        ]
        .spacing(0)
        .padding([2, 5]),
    )
    .style(|_: &Theme| container::Style {
        background: Some(MENU_BAR_BG.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

fn view_menu_button<'a>(
    label: &'a str,
    menu_type: MenuState,
    message: Message,
    current_menu: &MenuState,
) -> Element<'a, Message> {
    let is_active = *current_menu == menu_type;
    let hover_message = Message::MenuHovered(menu_type.clone());

    let btn = button(text(label).font(PRETENDARD))
        .padding([5.0, 10.0])
        .on_press(message)
        .style(move |_: &Theme, status: button::Status| {
            let bg_color = if is_active || status == button::Status::Hovered {
                MENU_BUTTON_HOVER_BG
            } else {
                MENU_BAR_BG
            };

            button::Style {
                background: Some(bg_color.into()),
                ..Default::default()
            }
        });

    mouse_area(btn).on_enter(hover_message).into()
}