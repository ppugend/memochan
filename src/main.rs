#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod config;
mod editor;
mod theme;

use crate::app::Notepad;
use crate::config::{PRETENDARD_FONT, PRETENDARD_FONT_NAME};
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MemoChan",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                PRETENDARD_FONT_NAME.to_owned(),
                std::sync::Arc::new(egui::FontData::from_static(PRETENDARD_FONT)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, PRETENDARD_FONT_NAME.to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(Notepad::new(cc)))
        }),
    )
}
