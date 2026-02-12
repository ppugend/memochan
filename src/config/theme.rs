use iced::{Color, Theme};

pub fn app_theme() -> Theme {
    Theme::Light
}

pub const MENU_BAR_BG: Color = Color::from_rgb(0.95, 0.95, 0.95);
pub const MENU_BUTTON_HOVER_BG: Color = Color::from_rgb(0.85, 0.85, 0.85);
pub const DROPDOWN_BG: Color = Color::WHITE;
pub const DROPDOWN_BORDER_COLOR: Color = Color::from_rgb(0.8, 0.8, 0.8);
pub const MENU_ITEM_HOVER_BG: Color = Color::from_rgb(0.9, 0.9, 0.95);
pub const SEPARATOR_COLOR: Color = Color::from_rgb(0.8, 0.8, 0.8);
pub const STATUS_BAR_BG: Color = Color::from_rgb(0.95, 0.95, 0.95);