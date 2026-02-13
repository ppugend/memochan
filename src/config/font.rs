pub const PRETENDARD_FONT: &[u8] = include_bytes!("../../assets/fonts/PretendardVariable.ttf");
pub const DEFAULT_FONT_SIZE: f32 = 14.0;
pub const MIN_FONT_SIZE: f32 = 8.0;
pub const DEFAULT_ZOOM_LEVEL: i16 = 100;
pub const MAX_ZOOM_LEVEL: i16 = 500;
pub const MIN_ZOOM_LEVEL: i16 = 50;

pub fn calculate_font_size(zoom_level: i16) -> f32 {
    (DEFAULT_FONT_SIZE * zoom_level as f32 / 100.0).max(MIN_FONT_SIZE)
}
