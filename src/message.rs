use iced::{font, widget::text_editor};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    FontLoaded(Result<(), font::Error>),

    NewFile,
    OpenFile,
    SaveFile,
    SaveFileAs,
    Exit,

    Undo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    TimeDate,

    WordWrapToggled,

    ZoomIn,
    ZoomOut,
    ZoomReset,
    StatusBarToggled,

    About,
    CloseAboutDialog,

    EditorAction(text_editor::Action),

    FileOpened(Result<(PathBuf, String), String>),
    FileSaved(Result<PathBuf, String>),

    FileMenuToggled,
    EditMenuToggled,
    FormatMenuToggled,
    ViewMenuToggled,
    HelpMenuToggled,
    CloseAllMenus,

    Event(iced::Event),

    ClipboardPaste(Option<String>),

    MouseScroll(f32),

    MenuHovered(MenuState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    None,
    File,
    Edit,
    Format,
    View,
    Help,
}

impl Default for MenuState {
    fn default() -> Self {
        Self::None
    }
}