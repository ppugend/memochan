use crate::config::{
    app_theme, calculate_font_size, DEFAULT_FONT_SIZE, DEFAULT_ZOOM_LEVEL, MAX_ZOOM_LEVEL,
    MIN_ZOOM_LEVEL, PRETENDARD_FONT,
};
use crate::message::{MenuState, Message};
use crate::view::main_view;
use chrono::Local;
use iced::{clipboard, font, keyboard, widget::text_editor, window, Element, Subscription, Task, Theme};
use iced::keyboard::Key;
use rfd::FileDialog;
use std::path::PathBuf;

pub struct Notepad {
    pub content: text_editor::Content,
    pub current_file: Option<PathBuf>,
    pub is_modified: bool,
    pub word_wrap: bool,
    pub font_size: f32,
    pub show_status_bar: bool,
    pub zoom_level: i16,
    pub menu_state: MenuState,
    pub show_about_dialog: bool,
    pub ctrl_pressed: bool,
}

impl Default for Notepad {
    fn default() -> Self {
        Self {
            content: text_editor::Content::new(),
            current_file: None,
            is_modified: false,
            word_wrap: true,
            font_size: DEFAULT_FONT_SIZE,
            show_status_bar: true,
            zoom_level: DEFAULT_ZOOM_LEVEL,
            menu_state: MenuState::default(),
            show_about_dialog: false,
            ctrl_pressed: false,
        }
    }
}

impl Notepad {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::batch(vec![
                font::load(PRETENDARD_FONT).map(Message::FontLoaded),
                window::set_min_size(window::Id::unique(), Some(iced::Size::new(400.0, 300.0))),
            ]),
        )
    }

    pub fn title(&self) -> String {
        let title = match &self.current_file {
            Some(path) => path.file_name().unwrap().to_string_lossy().to_string(),
            None => "Untitled".to_string(),
        };

        if self.is_modified {
            format!("*{} - MemoChan", title)
        } else {
            format!("{} - MemoChan", title)
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FontLoaded(result) => {
                if let Err(e) = result {
                    eprintln!("Failed to load font: {:?}", e);
                }
            }

            Message::NewFile => {
                self.content = text_editor::Content::new();
                self.current_file = None;
                self.is_modified = false;
                self.menu_state = MenuState::None;
            }
            Message::OpenFile => {
                self.menu_state = MenuState::None;
                return Task::perform(
                    async {
                        let file = FileDialog::new()
                            .add_filter("Text Documents", &["txt"])
                            .add_filter("All Files", &["*"])
                            .pick_file();

                        match file {
                            Some(path) => match std::fs::read_to_string(&path) {
                                Ok(content) => Ok((path, content)),
                                Err(e) => Err(format!("Cannot open file: {}", e)),
                            },
                            None => Err("No file selected".to_string()),
                        }
                    },
                    Message::FileOpened,
                );
            }
            Message::SaveFile => {
                if let Some(path) = self.current_file.clone() {
                    let content = self.content.text().to_string();
                    return Task::perform(
                        async move {
                            match std::fs::write(&path, &content) {
                                Ok(_) => Ok(path),
                                Err(e) => Err(format!("Cannot save file: {}", e)),
                            }
                        },
                        Message::FileSaved,
                    );
                } else {
                    return self.save_file_as();
                }
            }
            Message::SaveFileAs => {
                self.menu_state = MenuState::None;
                return self.save_file_as();
            }
            Message::Exit => {
                std::process::exit(0);
            }

            Message::Undo => {
                self.menu_state = MenuState::None;
            }
            Message::Cut => {
                let selected_text = self.get_selected_text();
                if let Some(text) = selected_text.clone() {
                    self.content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    self.is_modified = true;
                    self.menu_state = MenuState::None;
                    return clipboard::write(text);
                }
                self.menu_state = MenuState::None;
            }
            Message::Copy => {
                let selected_text = self.get_selected_text();
                if let Some(text) = selected_text.clone() {
                    self.menu_state = MenuState::None;
                    return clipboard::write(text);
                }
                self.menu_state = MenuState::None;
            }
            Message::Paste => {
                self.menu_state = MenuState::None;
                return clipboard::read().map(Message::ClipboardPaste);
            }
            Message::Delete => {
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.is_modified = true;
                self.menu_state = MenuState::None;
            }
            Message::SelectAll => {
                self.content.perform(text_editor::Action::SelectAll);
                self.menu_state = MenuState::None;
            }
            Message::TimeDate => {
                let now = Local::now().format("%Y-%m-%d %H:%M");
                let timestamp = format!("{}", now);
                for c in timestamp.chars() {
                    self.content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Insert(c)));
                }
                self.is_modified = true;
                self.menu_state = MenuState::None;
            }

            Message::WordWrapToggled => {
                self.word_wrap = !self.word_wrap;
                self.menu_state = MenuState::None;
            }

            Message::ZoomIn => {
                if self.zoom_level < MAX_ZOOM_LEVEL {
                    self.zoom_level += 10;
                    self.font_size = calculate_font_size(self.zoom_level);
                }
                self.menu_state = MenuState::None;
            }
            Message::ZoomOut => {
                if self.zoom_level > MIN_ZOOM_LEVEL {
                    self.zoom_level -= 10;
                    self.font_size = calculate_font_size(self.zoom_level);
                }
                self.menu_state = MenuState::None;
            }
            Message::ZoomReset => {
                self.zoom_level = DEFAULT_ZOOM_LEVEL;
                self.font_size = DEFAULT_FONT_SIZE;
                self.menu_state = MenuState::None;
            }
            Message::StatusBarToggled => {
                self.show_status_bar = !self.show_status_bar;
                self.menu_state = MenuState::None;
            }

            Message::About => {
                self.menu_state = MenuState::None;
                self.show_about_dialog = true;
            }
            Message::CloseAboutDialog => {
                self.show_about_dialog = false;
            }

            Message::EditorAction(action) => {
                let old_text = self.content.text().to_string();
                self.content.perform(action);
                let new_text = self.content.text().to_string();

                if old_text != new_text {
                    self.is_modified = true;
                }
            }

            Message::FileOpened(result) => match result {
                Ok((path, content)) => {
                    self.current_file = Some(path);
                    self.content = text_editor::Content::with_text(&content);
                    self.is_modified = false;
                }
                Err(e) => {
                    eprintln!("Error opening file: {}", e);
                }
            },
            Message::FileSaved(result) => match result {
                Ok(path) => {
                    self.current_file = Some(path);
                    self.is_modified = false;
                }
                Err(e) => {
                    eprintln!("Error saving file: {}", e);
                }
            },

            Message::FileMenuToggled => {
                self.menu_state = if self.menu_state == MenuState::File {
                    MenuState::None
                } else {
                    MenuState::File
                };
            }
            Message::EditMenuToggled => {
                self.menu_state = if self.menu_state == MenuState::Edit {
                    MenuState::None
                } else {
                    MenuState::Edit
                };
            }
            Message::FormatMenuToggled => {
                self.menu_state = if self.menu_state == MenuState::Format {
                    MenuState::None
                } else {
                    MenuState::Format
                };
            }
            Message::ViewMenuToggled => {
                self.menu_state = if self.menu_state == MenuState::View {
                    MenuState::None
                } else {
                    MenuState::View
                };
            }
            Message::HelpMenuToggled => {
                self.menu_state = if self.menu_state == MenuState::Help {
                    MenuState::None
                } else {
                    MenuState::Help
                };
            }
            Message::CloseAllMenus => {
                self.menu_state = MenuState::None;
            }

            Message::Event(event) => {
                match event {
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key, modifiers, ..
                    }) => {
                        let ctrl = modifiers.control() || modifiers.logo();
                        self.ctrl_pressed = ctrl;

                        if key == Key::Named(iced::keyboard::key::Named::Escape) {
                            self.menu_state = MenuState::None;
                            return Task::none();
                        }

                        if ctrl {
                            match key {
                                Key::Character(c) if c == "n" || c == "N" => {
                                    return self.update(Message::NewFile);
                                }
                                Key::Character(c) if c == "o" || c == "O" => {
                                    return self.update(Message::OpenFile);
                                }
                                Key::Character(c) if c == "s" || c == "S" => {
                                    return self.update(Message::SaveFile);
                                }
                                Key::Character(c) if c == "+" || c == "=" => {
                                    return self.update(Message::ZoomIn);
                                }
                                Key::Character(c) if c == "-" => {
                                    return self.update(Message::ZoomOut);
                                }
                                Key::Character(c) if c == "0" => {
                                    return self.update(Message::ZoomReset);
                                }
                                Key::Character(c) if c == "a" || c == "A" => {
                                    return self.update(Message::SelectAll);
                                }
                                Key::Character(c) if c == "z" || c == "Z" => {
                                    return self.update(Message::Undo);
                                }
                                Key::Character(c) if c == "x" || c == "X" => {
                                    return self.update(Message::Cut);
                                }
                                Key::Character(c) if c == "c" || c == "C" => {
                                    return self.update(Message::Copy);
                                }
                                Key::Character(c) if c == "v" || c == "V" => {
                                    return self.update(Message::Paste);
                                }
                                _ => {}
                            }
                        }

                        match key {
                            Key::Named(iced::keyboard::key::Named::F5) => {
                                return self.update(Message::TimeDate);
                            }
                            Key::Named(iced::keyboard::key::Named::Delete) => {
                                return self.update(Message::Delete);
                            }
                            _ => {}
                        }
                    }
                    iced::Event::Keyboard(keyboard::Event::KeyReleased { modifiers, .. }) => {
                        self.ctrl_pressed = modifiers.control() || modifiers.logo();
                    }
                    iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                        if self.ctrl_pressed {
                            match delta {
                                iced::mouse::ScrollDelta::Lines { y, .. }
                                | iced::mouse::ScrollDelta::Pixels { y, .. } => {
                                    if y != 0.0 {
                                        return self.update(Message::MouseScroll(y));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            Message::ClipboardPaste(maybe_text) => {
                if let Some(text) = maybe_text {
                    for c in text.chars() {
                        self.content
                            .perform(text_editor::Action::Edit(text_editor::Edit::Insert(c)));
                    }
                    self.is_modified = true;
                }
            }

            Message::MouseScroll(delta) => {
                if delta > 0.0 {
                    if self.zoom_level < MAX_ZOOM_LEVEL {
                        self.zoom_level = (self.zoom_level + 10).min(MAX_ZOOM_LEVEL);
                        self.font_size = calculate_font_size(self.zoom_level);
                    }
                } else {
                    if self.zoom_level > MIN_ZOOM_LEVEL {
                        self.zoom_level = (self.zoom_level - 10).max(MIN_ZOOM_LEVEL);
                        self.font_size = calculate_font_size(self.zoom_level);
                    }
                }
            }

            Message::MenuHovered(hovered_menu) => {
                if self.menu_state != MenuState::None && self.menu_state != hovered_menu {
                    self.menu_state = hovered_menu;
                }
            }
        }

        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(Message::Event)
    }

    pub fn theme(&self) -> Theme {
        app_theme()
    }

    pub fn view(&self) -> Element<'_, Message> {
        main_view(self)
    }

    fn save_file_as(&mut self) -> Task<Message> {
        let content = self.content.text().to_string();
        Task::perform(
            async move {
                let file = FileDialog::new()
                    .add_filter("Text Documents", &["txt"])
                    .add_filter("All Files", &["*"])
                    .save_file();

                match file {
                    Some(path) => match std::fs::write(&path, &content) {
                        Ok(_) => Ok(path),
                        Err(e) => Err(format!("Cannot save file: {}", e)),
                    },
                    None => Err("No file selected".to_string()),
                }
            },
            Message::FileSaved,
        )
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        let text = self.content.text();
        let mut line: usize = 1;
        let mut column: usize = 1;

        for c in text.chars() {
            if c == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        (line, column.saturating_sub(1).max(1))
    }

    fn get_selected_text(&self) -> Option<String> {
        self.content.selection()
    }
}