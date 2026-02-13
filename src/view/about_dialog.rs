use crate::config::{APP_ICON, PRETENDARD};
use crate::message::Message;
use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Alignment, Border, Element, Length};
use std::sync::OnceLock;

static APP_ICON_HANDLE: OnceLock<image::Handle> = OnceLock::new();

fn get_app_icon_handle() -> image::Handle {
    APP_ICON_HANDLE
        .get_or_init(|| image::Handle::from_bytes(APP_ICON))
        .clone()
}

pub fn view_about_dialog() -> Element<'static, Message> {
    let overlay = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4).into()),
            ..Default::default()
        });

    let icon = image(get_app_icon_handle())
        .width(Length::Fixed(80.0))
        .height(Length::Fixed(80.0));

    let title_text = text("MemoChan")
        .font(PRETENDARD)
        .size(20)
        .style(|_: &iced::Theme| text::Style {
            color: Some(iced::Color::from_rgb(0.1, 0.1, 0.1)),
        });

    let version_text = text(format!("Version {}", env!("CARGO_PKG_VERSION")))
        .font(PRETENDARD)
        .size(11)
        .style(|_: &iced::Theme| text::Style {
            color: Some(iced::Color::from_rgb(0.4, 0.4, 0.4)),
        });

    let title_column = column![title_text, Space::new().height(2), version_text]
        .align_x(Alignment::Start);

    let separator_line = container(Space::new())
        .width(Length::Fill)
        .height(1.0)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Color::from_rgb(0.85, 0.85, 0.85).into()),
            ..Default::default()
        });

    let info_text = text(format!(
        "A simple notepad application built with Iced {}\nAuthor: {}\nLicense: {}",
        "0.14",
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_LICENSE")
    ))
    .font(PRETENDARD)
    .size(11)
    .style(|_: &iced::Theme| text::Style {
        color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
    });

    let close_button = button(
        container(text("OK").font(PRETENDARD).size(12))
            .width(Length::Fixed(80.0))
            .align_x(Alignment::Center),
    )
    .on_press(Message::CloseAboutDialog)
    .padding([6, 16])
    .style(|_, status| {
        let (bg, border_color) = match status {
            button::Status::Hovered => (
                iced::Color::from_rgb(0.95, 0.95, 0.95),
                iced::Color::from_rgb(0.3, 0.6, 0.9),
            ),
            button::Status::Pressed => (
                iced::Color::from_rgb(0.9, 0.9, 0.9),
                iced::Color::from_rgb(0.2, 0.5, 0.8),
            ),
            _ => (
                iced::Color::from_rgb(0.98, 0.98, 0.98),
                iced::Color::from_rgb(0.7, 0.7, 0.7),
            ),
        };
        button::Style {
            background: Some(bg.into()),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        }
    });

    let header_row = row![
        icon,
        Space::new().width(12),
        title_column,
    ]
    .align_y(Alignment::Center);

    let dialog_content = column![
        header_row,
        Space::new().height(12),
        separator_line,
        Space::new().height(12),
        column![info_text].align_x(Alignment::Center),
        Space::new().height(16),
        close_button,
    ]
    .align_x(Alignment::Center)
    .padding(20);

    let dialog = container(dialog_content)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Color::from_rgb(0.98, 0.98, 0.98).into()),
            border: Border {
                color: iced::Color::from_rgb(0.6, 0.6, 0.6),
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        })
        .width(Length::Fixed(300.0))
        .height(Length::Shrink);

    let centered_dialog = container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    iced::widget::Stack::new()
        .push(overlay)
        .push(centered_dialog)
        .into()
}