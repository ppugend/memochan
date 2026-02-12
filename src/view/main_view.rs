use crate::app::Notepad;
use crate::config::PRETENDARD;
use crate::message::Message;
use crate::view::about_dialog::view_about_dialog;
use crate::view::dropdown::view_dropdown;
use crate::view::menu_bar::view_menu_bar;
use crate::view::status_bar::view_status_bar;
use iced::widget::{container, mouse_area, text_editor, Stack, Space, text::Wrapping};
use iced::{Element, Length};

pub fn main_view(notepad: &Notepad) -> Element<'_, Message> {
    let menu_bar = view_menu_bar(notepad);

    let editor = if notepad.word_wrap {
        text_editor(&notepad.content)
            .on_action(Message::EditorAction)
            .font(PRETENDARD)
            .size(notepad.font_size)
            .padding(10)
            .wrapping(Wrapping::Word)
    } else {
        text_editor(&notepad.content)
            .on_action(Message::EditorAction)
            .font(PRETENDARD)
            .size(notepad.font_size)
            .padding(10)
            .wrapping(Wrapping::None)
    };

    let mut main_layout = iced::widget::column![menu_bar].spacing(0).width(Length::Fill);

    main_layout = main_layout.push(container(editor).width(Length::Fill).height(Length::Fill));

    if notepad.show_status_bar {
        main_layout = main_layout.push(view_status_bar(notepad));
    }

    let base_content = container(main_layout)
        .width(Length::Fill)
        .height(Length::Fill);

    if notepad.show_about_dialog {
        let base_with_dialog = Stack::new()
            .push(base_content)
            .push(view_about_dialog());
        base_with_dialog.into()
    } else if notepad.menu_state != crate::message::MenuState::None {
        let dropdown = view_dropdown(notepad);

        let click_overlay = mouse_area(container::Container::new(Space::new())
            .width(Length::Fill)
            .height(Length::Fill))
            .on_press(Message::CloseAllMenus);

        Stack::new()
            .push(base_content)
            .push(click_overlay)
            .push(dropdown)
            .into()
    } else {
        base_content.into()
    }
}