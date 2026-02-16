use egui::{Color32, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }

    pub fn is_dark(&self, system_dark: bool) -> bool {
        match self {
            ThemeMode::System => system_dark,
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            ThemeMode::System => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
        }
    }
}

#[allow(dead_code)]
pub struct ThemeColors {
    pub background: Color32,
    pub text: Color32,
    pub dim_text: Color32,
    pub accent: Color32,
    pub selection: Color32,
    pub line_number: Color32,
    pub line_number_bg: Color32,
    pub gutter_bg: Color32,
    pub current_line_bg: Color32,
}

#[allow(dead_code)]
impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: Color32::from_rgb(0x1e, 0x1e, 0x1e),
            text: Color32::from_rgb(0xd4, 0xd4, 0xd4),
            dim_text: Color32::from_rgb(0x80, 0x80, 0x80),
            accent: Color32::from_rgb(0x56, 0x9c, 0xd6),
            selection: Color32::from_rgba_unmultiplied(0x26, 0x4f, 0x78, 0x80),
            line_number: Color32::from_rgb(0x85, 0x85, 0x85),
            line_number_bg: Color32::from_rgb(0x1e, 0x1e, 0x1e),
            gutter_bg: Color32::from_rgb(0x25, 0x25, 0x26),
            current_line_bg: Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 0x08),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color32::from_rgb(0xff, 0xff, 0xff),
            text: Color32::from_rgb(0x33, 0x33, 0x33),
            dim_text: Color32::from_rgb(0x80, 0x80, 0x80),
            accent: Color32::from_rgb(0x00, 0x66, 0xcc),
            selection: Color32::from_rgba_unmultiplied(0xad, 0xd6, 0xff, 0x80),
            line_number: Color32::from_rgb(0x9e, 0x9e, 0x9e),
            line_number_bg: Color32::from_rgb(0xff, 0xff, 0xff),
            gutter_bg: Color32::from_rgb(0xf5, 0xf5, 0xf5),
            current_line_bg: Color32::from_rgba_unmultiplied(0x00, 0x00, 0x00, 0x08),
        }
    }

    pub fn for_mode(mode: ThemeMode, system_dark: bool) -> Self {
        if mode.is_dark(system_dark) {
            Self::dark()
        } else {
            Self::light()
        }
    }
}

pub fn apply_egui_style(style: &mut Style, dark: bool) {
    if dark {
        style.visuals = egui::Visuals::dark();
    } else {
        style.visuals = egui::Visuals::light();
    }
}
