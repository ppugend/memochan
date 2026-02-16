use crate::config::{
    calculate_font_size, APP_ICON, DEFAULT_FONT_SIZE, DEFAULT_ZOOM_LEVEL, MAX_ZOOM_LEVEL,
    MIN_ZOOM_LEVEL,
};
use crate::editor::SyntaxHighlighter;
use crate::theme::{apply_egui_style, ThemeMode};
use chrono::Local;
use eframe::egui;
use egui::{ColorImage, FontId, RichText, TextureHandle};
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PreviewLayout {
    #[default]
    Hidden,
    Tabs,
    Horizontal,
}

impl PreviewLayout {
    pub fn name(&self) -> &'static str {
        match self {
            PreviewLayout::Hidden => "Off",
            PreviewLayout::Tabs => "Tabs",
            PreviewLayout::Horizontal => "Side by Side",
        }
    }

    pub fn is_visible(&self) -> bool {
        *self != PreviewLayout::Hidden
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EditorTab {
    #[default]
    Editor,
    Preview,
}

#[derive(Clone, Debug)]
struct SearchResult {
    index: usize,
    flash_timer: f32,
}

#[derive(Clone, Debug)]
enum FdAct {
    Open,
    SaveAs(String),
}

pub struct Notepad {
    text: String,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    last_saved: String,
    file: Option<PathBuf>,
    modified: bool,
    word_wrap: bool,
    font_size: f32,
    zoom: i16,
    status_bar: bool,
    about: bool,
    confirm: bool,
    confirm_act: ConfirmAct,
    cur_line: usize,
    cur_col: usize,
    fd_rx: Option<Receiver<Option<PathBuf>>>,
    fd_act: Option<FdAct>,
    icon: Option<TextureHandle>,
    first_frame: bool,

    theme_mode: ThemeMode,
    system_dark: bool,
    preview_layout: PreviewLayout,
    active_tab: EditorTab,
    editor_focus_request: bool,
    preview_focus_request: bool,

    highlighter: SyntaxHighlighter,
    current_syntax: Option<String>,
    auto_detect: bool,
    context_menu: bool,
    all_languages: Vec<String>,

    markdown_cache: egui_commonmark::CommonMarkCache,
    editor_scroll_offset: f32,
    editor_scroll_max: f32,
    preview_scroll_offset: f32,
    preview_scroll_max: f32,
    syncing_scroll: bool,

    search_query: String,
    search_active: bool,
    search_results: Vec<SearchResult>,
    current_search_idx: usize,
    search_focus: bool,
    search_select_all: bool,
    search_input_has_focus: bool,
    search_cursor_pos: Option<usize>,

    notification_text: String,
    notification_timer: f32,

    pending_large_file: Option<PathBuf>,
    large_file_confirm: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ConfirmAct {
    None,
    New,
    Open,
    Exit,
}

#[cfg(target_os = "macos")]
fn is_cmd(m: &egui::Modifiers) -> bool {
    m.mac_cmd
}

#[cfg(not(target_os = "macos"))]
fn is_cmd(m: &egui::Modifiers) -> bool {
    m.ctrl
}

#[cfg(target_os = "macos")]
fn mod_key() -> &'static str {
    "Cmd"
}

#[cfg(not(target_os = "macos"))]
fn mod_key() -> &'static str {
    "Ctrl"
}

impl Default for Notepad {
    fn default() -> Self {
        let highlighter = SyntaxHighlighter::new();
        let all_languages: Vec<String> = highlighter
            .all_syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        Self {
            text: String::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_saved: String::new(),
            file: None,
            modified: false,
            word_wrap: true,
            font_size: DEFAULT_FONT_SIZE,
            zoom: DEFAULT_ZOOM_LEVEL,
            status_bar: true,
            about: false,
            confirm: false,
            confirm_act: ConfirmAct::None,
            cur_line: 1,
            cur_col: 1,
            fd_rx: None,
            fd_act: None,
            icon: None,
            first_frame: true,
            theme_mode: ThemeMode::default(),
            system_dark: true,
            preview_layout: PreviewLayout::default(),
            active_tab: EditorTab::default(),
            editor_focus_request: false,
            preview_focus_request: false,
            highlighter,
            current_syntax: None,
            auto_detect: true,
            context_menu: false,
            all_languages,
            markdown_cache: egui_commonmark::CommonMarkCache::default(),
            editor_scroll_offset: 0.0,
            editor_scroll_max: 0.0,
            preview_scroll_offset: 0.0,
            preview_scroll_max: 0.0,
            syncing_scroll: false,
            search_query: String::new(),
            search_active: false,
            search_results: Vec::new(),
            current_search_idx: 0,
            search_focus: false,
            search_select_all: false,
            search_input_has_focus: false,
            search_cursor_pos: None,
            notification_text: String::new(),
            notification_timer: 0.0,
            pending_large_file: None,
            large_file_confirm: false,
        }
    }
}

impl Notepad {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        if let Ok(img) = ::image::load_from_memory(APP_ICON) {
            let img = img.to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            let pixels: Vec<egui::Color32> = img
                .pixels()
                .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                .collect();
            app.icon = Some(cc.egui_ctx.load_texture(
                "icon",
                ColorImage { size, pixels },
                Default::default(),
            ));
        }
        app.system_dark = cc.egui_ctx.style().visuals.dark_mode;
        app.apply_theme(&cc.egui_ctx);
        app
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        let dark = self.theme_mode.is_dark(self.system_dark);
        let mut style = (*ctx.style()).clone();
        apply_egui_style(&mut style, dark);
        ctx.set_style(style);
    }

    fn is_dark(&self) -> bool {
        self.theme_mode.is_dark(self.system_dark)
    }

    fn is_markdown(&self) -> bool {
        self.current_syntax.as_deref() == Some("Markdown")
            || self
                .file
                .as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase() == "md")
                .unwrap_or(false)
    }

    fn auto_detect_language(&mut self) {
        if let Some(ref path) = self.file {
            if let Some(syntax) = self
                .highlighter
                .find_syntax_by_extension(path.extension().and_then(|e| e.to_str()).unwrap_or(""))
            {
                self.current_syntax = Some(syntax.name.clone());
                return;
            }
        }
        self.current_syntax = None;
    }

    fn set_manual_language(&mut self, name: &str) {
        self.auto_detect = false;
        self.current_syntax = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
    }

    fn set_auto_mode(&mut self) {
        self.auto_detect = true;
        self.auto_detect_language();
    }

    fn update_preview_state(&mut self) {
        if self.is_markdown() && !self.preview_layout.is_visible() {
            self.preview_layout = PreviewLayout::Tabs;
            self.active_tab = EditorTab::Editor;
        } else if !self.is_markdown() && self.preview_layout.is_visible() {
            self.preview_layout = PreviewLayout::Hidden;
        }
    }

    fn toggle_preview_tab(&mut self) {
        if self.preview_layout.is_visible() {
            self.active_tab = match self.active_tab {
                EditorTab::Editor => {
                    self.preview_focus_request = true;
                    EditorTab::Preview
                }
                EditorTab::Preview => {
                    self.editor_focus_request = true;
                    EditorTab::Editor
                }
            };
        }
    }

    fn perform_search(&mut self) {
        self.search_results.clear();
        if self.search_query.is_empty() {
            self.current_search_idx = 0;
            return;
        }
        let query = self.search_query.to_lowercase();
        let text_lower = self.text.to_lowercase();
        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&query) {
            let abs_pos = start + pos;
            self.search_results.push(SearchResult {
                index: abs_pos,
                flash_timer: 0.0,
            });
            start = abs_pos + 1;
        }
        self.current_search_idx = 0;
        if !self.search_results.is_empty() {
            self.search_results[0].flash_timer = 1.0;
            self.scroll_to_search_result();
        }
    }

    fn scroll_to_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        let result = &self.search_results[self.current_search_idx];
        let line_count = self.text[..result.index].matches('\n').count();
        let line_height = self.font_size * 1.5;
        self.editor_scroll_offset = (line_count as f32 * line_height).max(0.0);
        self.syncing_scroll = true;
    }

    fn flash_current_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        self.search_results[self.current_search_idx].flash_timer = 1.0;
    }

    fn next_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        self.current_search_idx = (self.current_search_idx + 1) % self.search_results.len();
        self.scroll_to_search_result();
        self.flash_current_result();
    }

    fn prev_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        self.current_search_idx = if self.current_search_idx == 0 {
            self.search_results.len() - 1
        } else {
            self.current_search_idx - 1
        };
        self.scroll_to_search_result();
        self.flash_current_result();
    }

    fn show_notification(&mut self, text: &str) {
        self.notification_text = text.to_string();
        self.notification_timer = 3.0;
    }

    fn show_notification_toast(&self, ctx: &egui::Context) {
        let toast_width = 300.0;
        let toast_height = 40.0;
        let margin = 20.0;

        egui::Area::new(egui::Id::new("notification_toast"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-margin, margin))
            .show(ctx, |ui| {
                let frame = egui::Frame::popup(ui.style()).fill(if self.is_dark() {
                    egui::Color32::from_rgb(0x2d, 0x2d, 0x2d)
                } else {
                    egui::Color32::from_rgb(0xf5, 0xf5, 0xf5)
                });
                frame.show(ui, |ui| {
                    ui.set_min_size(egui::Vec2::new(toast_width, toast_height));
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&self.notification_text).size(14.0));
                    });
                });
            });
    }

    fn save_undo(&mut self) {
        if self.text != self.last_saved {
            self.undo_stack.push(self.last_saved.clone());
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.last_saved = self.text.clone();
        }
    }

    fn undo(&mut self) {
        if let Some(p) = self.undo_stack.pop() {
            self.redo_stack.push(self.text.clone());
            self.text = p;
            self.last_saved = self.text.clone();
            self.upd_mod();
        }
    }

    fn redo(&mut self) {
        if let Some(n) = self.redo_stack.pop() {
            self.undo_stack.push(self.text.clone());
            self.text = n;
            self.last_saved = self.text.clone();
            self.upd_mod();
        }
    }

    fn upd_mod(&mut self) {
        self.modified = self.file.is_none()
            || std::fs::read_to_string(self.file.as_ref().unwrap())
                .map(|c| c != self.text)
                .unwrap_or(true);
    }

    fn check_fd(&mut self) {
        if let Some(rx) = &self.fd_rx {
            if let Ok(Some(p)) = rx.try_recv() {
                match &self.fd_act {
                    Some(FdAct::Open) => {
                        if let Ok(metadata) = std::fs::metadata(&p) {
                            let size_kb = metadata.len() / 1024;
                            if size_kb > 500 {
                                self.pending_large_file = Some(p);
                                self.large_file_confirm = true;
                            } else {
                                match std::fs::read_to_string(&p) {
                                    Ok(c) => self.load_file_content(c, p),
                                    Err(e) => {
                                        self.show_notification(&format!("Cannot open file: {}", e));
                                    }
                                }
                            }
                        } else {
                            match std::fs::read_to_string(&p) {
                                Ok(c) => self.load_file_content(c, p),
                                Err(e) => {
                                    self.show_notification(&format!("Cannot open file: {}", e));
                                }
                            }
                        }
                    }
                    Some(FdAct::SaveAs(c)) => {
                        let final_path = self.fix_extension(&p, c);
                        if std::fs::write(&final_path, c).is_ok() {
                            self.file = Some(final_path);
                            self.modified = false;
                            if self.auto_detect {
                                self.auto_detect_language();
                            }
                        }
                    }
                    None => {}
                }
                self.fd_rx = None;
                self.fd_act = None;
            }
        }
    }

    fn load_file_content(&mut self, content: String, path: PathBuf) {
        self.text = content;
        self.last_saved = self.text.clone();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.file = Some(path);
        self.modified = false;
        self.auto_detect = true;
        self.auto_detect_language();
        self.editor_scroll_offset = 0.0;
        self.search_query.clear();
        self.search_results.clear();
        self.search_active = false;
        self.search_cursor_pos = None;
        self.update_preview_state();
        self.active_tab = EditorTab::Editor;
    }

    fn fix_extension(&self, path: &PathBuf, _content: &str) -> PathBuf {
        let default_ext = self.get_default_extension();

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == default_ext {
                return path.clone();
            }
            if ext == "txt" && default_ext != "txt" {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled");
                return path.with_file_name(format!("{}.{}", stem, default_ext));
            }
        }

        path.clone()
    }

    fn req_new(&mut self) {
        if self.modified {
            self.confirm_act = ConfirmAct::New;
            self.confirm = true;
        } else {
            self.do_new();
        }
    }

    fn do_new(&mut self) {
        self.text.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.last_saved.clear();
        self.file = None;
        self.modified = false;
        self.confirm = false;
        self.confirm_act = ConfirmAct::None;
        self.current_syntax = None;
        self.auto_detect = true;
        self.preview_layout = PreviewLayout::Hidden;
        self.editor_scroll_offset = 0.0;
        self.search_query.clear();
        self.search_results.clear();
        self.search_active = false;
    }

    fn req_open(&mut self) {
        if self.modified {
            self.confirm_act = ConfirmAct::Open;
            self.confirm = true;
        } else {
            self.do_open();
        }
    }

    fn do_open(&mut self) {
        self.confirm = false;
        self.confirm_act = ConfirmAct::None;
        let (tx, rx) = mpsc::channel();
        self.fd_rx = Some(rx);
        self.fd_act = Some(FdAct::Open);
        std::thread::spawn(move || {
            let _ = tx.send(FileDialog::new().pick_file());
        });
    }

    fn save(&mut self) {
        if let Some(ref p) = self.file {
            if std::fs::write(p, &self.text).is_ok() {
                self.modified = false;
            }
        } else {
            self.save_as();
        }
    }

    fn get_default_extension(&self) -> &'static str {
        if let Some(ref syntax) = self.current_syntax {
            match syntax.as_str() {
                "Markdown" => "md",
                "Rust" => "rs",
                "Python" => "py",
                "JavaScript" => "js",
                "TypeScript" => "ts",
                "Java" => "java",
                "C" => "c",
                "C++" => "cpp",
                "C#" => "cs",
                "Go" => "go",
                "Ruby" => "rb",
                "PHP" => "php",
                "HTML" => "html",
                "CSS" => "css",
                "JSON" => "json",
                "YAML" => "yaml",
                "TOML" => "toml",
                "Shell Script" => "sh",
                "SQL" => "sql",
                "Swift" => "swift",
                "Objective-C" => "m",
                "Kotlin" => "kt",
                _ => "txt",
            }
        } else {
            "txt"
        }
    }

    fn save_as(&mut self) {
        let c = self.text.clone();
        let default_ext = self.get_default_extension().to_string();
        let default_name = if self.file.is_none() {
            format!("untitled.{}", default_ext)
        } else {
            self.file
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(&format!("untitled.{}", default_ext))
                .to_string()
        };
        let (tx, rx) = mpsc::channel();
        self.fd_rx = Some(rx);
        self.fd_act = Some(FdAct::SaveAs(c));
        std::thread::spawn(move || {
            let _ = tx.send(FileDialog::new().set_file_name(default_name).save_file());
        });
    }

    fn conf_save(&mut self) {
        self.confirm = false;
        if self.file.is_some() {
            self.save();
            self.exec_conf();
        } else {
            self.save_as();
        }
    }

    fn conf_nosave(&mut self) {
        self.confirm = false;
        self.exec_conf();
    }

    fn exec_conf(&mut self) {
        match self.confirm_act {
            ConfirmAct::New => self.do_new(),
            ConfirmAct::Open => self.do_open(),
            ConfirmAct::Exit => std::process::exit(0),
            ConfirmAct::None => {}
        }
    }

    fn ins_time(&mut self) {
        self.save_undo();
        self.text
            .push_str(&Local::now().format("%Y-%m-%d %H:%M").to_string());
        self.modified = true;
    }

    fn upd_cur(&mut self, o: &egui::text_edit::TextEditOutput) {
        if let Some(r) = o.cursor_range.clone() {
            let b: String = self.text.chars().take(r.primary.ccursor.index).collect();
            let v: Vec<&str> = b.split('\n').collect();
            self.cur_line = v.len();
            self.cur_col = v.last().map(|l| l.chars().count() + 1).unwrap_or(1);
        }
    }

    fn z_in(&mut self) {
        if self.zoom < MAX_ZOOM_LEVEL {
            self.zoom = (self.zoom + 10).min(MAX_ZOOM_LEVEL);
            self.font_size = calculate_font_size(self.zoom);
        }
    }

    fn z_out(&mut self) {
        if self.zoom > MIN_ZOOM_LEVEL {
            self.zoom = (self.zoom - 10).max(MIN_ZOOM_LEVEL);
            self.font_size = calculate_font_size(self.zoom);
        }
    }

    fn z_res(&mut self) {
        self.zoom = DEFAULT_ZOOM_LEVEL;
        self.font_size = DEFAULT_FONT_SIZE;
    }

    fn upd_title(&self, ctx: &egui::Context) {
        let title = if let Some(ref p) = self.file {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("Untitled");
            if self.modified {
                format!("*{} - MemoChan", name)
            } else {
                format!("{} - MemoChan", name)
            }
        } else {
            if self.modified {
                "*Untitled - MemoChan".to_string()
            } else {
                "Untitled - MemoChan".to_string()
            }
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }
}

impl eframe::App for Notepad {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.check_fd();
        self.upd_title(ctx);
        ctx.set_zoom_factor(1.0);

        let dark = self.is_dark();
        let mut zi = false;
        let mut zo = false;
        let mut zr = false;
        let mut toggle_tab = false;
        let mut toggle_search = false;

        if !self.about && !self.confirm {
            ctx.input(|i| {
                for e in &i.raw.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = e
                    {
                        if is_cmd(&modifiers) {
                            match key {
                                egui::Key::N => self.req_new(),
                                egui::Key::O => self.req_open(),
                                egui::Key::S => {
                                    if modifiers.shift {
                                        self.save_as();
                                    } else {
                                        self.save();
                                    }
                                }
                                egui::Key::Z => {
                                    if modifiers.shift {
                                        self.redo();
                                    } else {
                                        self.undo();
                                    }
                                }
                                egui::Key::Y => self.redo(),
                                egui::Key::P => toggle_tab = true,
                                egui::Key::F => toggle_search = true,
                                _ => {}
                            }
                        }
                        // Zoom shortcuts: Ctrl only (not Cmd on macOS)
                        if modifiers.ctrl && !modifiers.mac_cmd {
                            match key {
                                egui::Key::Plus | egui::Key::Equals => zi = true,
                                egui::Key::Minus => zo = true,
                                egui::Key::Num0 => zr = true,
                                _ => {}
                            }
                        }
                    }
                }
            });

            if self.search_active && self.search_input_has_focus {
                ctx.input(|i| {
                    for e in &i.raw.events {
                        if let egui::Event::Key {
                            key, pressed: true, ..
                        } = e
                        {
                            match key {
                                egui::Key::Escape => {
                                    if !self.search_results.is_empty() {
                                        let result = &self.search_results[self.current_search_idx];
                                        self.search_cursor_pos = Some(result.index);
                                    }
                                    self.search_active = false;
                                    if self.active_tab == EditorTab::Editor {
                                        self.editor_focus_request = true;
                                    } else {
                                        self.preview_focus_request = true;
                                    }
                                }
                                egui::Key::ArrowUp => self.prev_search_result(),
                                egui::Key::ArrowDown => self.next_search_result(),
                                _ => {}
                            }
                        }
                    }
                });
            }
        }

        if self.confirm {
            ctx.input(|i| {
                for e in &i.raw.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = e
                    {
                        match key {
                            egui::Key::S => self.conf_save(),
                            egui::Key::D => self.conf_nosave(),
                            egui::Key::Escape => {
                                self.confirm = false;
                                self.confirm_act = ConfirmAct::None;
                            }
                            _ => {}
                        }
                    }
                }
            });
        }

        if self.about {
            ctx.input(|i| {
                for e in &i.raw.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = e
                    {
                        if *key == egui::Key::Escape || *key == egui::Key::Enter {
                            self.about = false;
                        }
                    }
                }
            });
        }

        if zi {
            self.z_in();
        }
        if zo {
            self.z_out();
        }
        if zr {
            self.z_res();
        }
        if toggle_tab {
            self.toggle_preview_tab();
        }
        if toggle_search {
            if !self.search_active {
                self.search_active = true;
                self.search_focus = true;
                self.search_select_all = true;
                self.perform_search();
            } else {
                self.search_focus = true;
                self.search_select_all = true;
            }
        }

        egui::TopBottomPanel::top("menu").show(ctx, |ui| self.menu(ui));

        if self.search_active {
            egui::TopBottomPanel::top("search_bar")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("X").clicked() {
                            if !self.search_results.is_empty() {
                                let result = &self.search_results[self.current_search_idx];
                                self.search_cursor_pos = Some(result.index);
                            }
                            self.search_active = false;
                            if self.active_tab == EditorTab::Editor {
                                self.editor_focus_request = true;
                            } else {
                                self.preview_focus_request = true;
                            }
                        }
                        ui.label("Find:");

                        let search_input_id = egui::Id::new("search_input");
                        let mut output = egui::TextEdit::singleline(&mut self.search_query)
                            .desired_width(200.0)
                            .id(search_input_id)
                            .show(ui);

                        if self.search_focus {
                            output.response.request_focus();
                            self.search_focus = false;
                        }

                        if self.search_select_all {
                            let text_len = self.search_query.len();
                            output.state.cursor.set_char_range(Some(
                                egui::text::CCursorRange::two(
                                    egui::text::CCursor::new(0),
                                    egui::text::CCursor::new(text_len),
                                ),
                            ));
                            output.state.store(ui.ctx(), search_input_id);
                            self.search_select_all = false;
                        }

                        self.search_input_has_focus = output.response.has_focus();

                        if output.response.changed() {
                            self.perform_search();
                        }
                        if output.response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            self.next_search_result();
                            self.search_focus = true;
                        }

                        let count_text = if self.search_query.is_empty() {
                            "".to_string()
                        } else if self.search_results.is_empty() {
                            "No results".to_string()
                        } else {
                            format!(
                                "{}/{}",
                                self.current_search_idx + 1,
                                self.search_results.len()
                            )
                        };
                        ui.label(count_text);
                        if ui.button("↑").clicked() {
                            self.prev_search_result();
                        }
                        if ui.button("↓").clicked() {
                            self.next_search_result();
                        }
                    });
                });
        }

        if self.status_bar {
            egui::TopBottomPanel::bottom("status").show(ctx, |ui| self.status(ui, dark));
        }
        self.main_area(ctx, dark);
        if self.about {
            self.about_md(ctx);
        }
        if self.confirm {
            self.conf_md(ctx);
        }
        if self.context_menu {
            self.show_context_menu(ctx);
        }
        if self.large_file_confirm {
            self.large_file_confirm_md(ctx);
        }

        if !self.notification_text.is_empty() {
            self.show_notification_toast(ctx);
            self.notification_timer -= ctx.input(|i| i.stable_dt.min(0.1));
            if self.notification_timer <= 0.0 {
                self.notification_text.clear();
            }
        }

        for result in &mut self.search_results {
            if result.flash_timer > 0.0 {
                result.flash_timer -= ctx.input(|i| i.stable_dt.min(0.1));
            }
        }

        ctx.request_repaint();
    }
}

impl Notepad {
    fn menu(&mut self, ui: &mut egui::Ui) {
        let key = mod_key();
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button(format!("New\t{}+N", key)).clicked() {
                    self.req_new();
                    ui.close_menu();
                }
                if ui.button(format!("Open...\t{}+O", key)).clicked() {
                    self.req_open();
                    ui.close_menu();
                }
                if ui.button(format!("Save\t{}+S", key)).clicked() {
                    self.save();
                    ui.close_menu();
                }
                if ui.button(format!("Save As...\tShift+{}+S", key)).clicked() {
                    self.save_as();
                    ui.close_menu();
                }

                ui.separator();
                if ui.button("Exit").clicked() {
                    if self.modified {
                        self.confirm_act = ConfirmAct::Exit;
                        self.confirm = true;
                    } else {
                        std::process::exit(0);
                    }
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button(format!("Undo\t{}+Z", key)).clicked() {
                    self.undo();
                    ui.close_menu();
                }
                if ui.button(format!("Redo\t{}+Y", key)).clicked() {
                    self.redo();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(format!("Find\t{}+F", key)).clicked() {
                    self.search_active = true;
                    self.search_focus = true;
                    self.perform_search();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Time/Date\tF5").clicked() {
                    self.ins_time();
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Zoom In\tCtrl++").clicked() {
                    self.z_in();
                    ui.close_menu();
                }
                if ui.button("Zoom Out\tCtrl+-").clicked() {
                    self.z_out();
                    ui.close_menu();
                }
                if ui.button("Reset Zoom\tCtrl+0").clicked() {
                    self.z_res();
                    ui.close_menu();
                }
                ui.separator();
                let m = if self.status_bar { "✓ " } else { "  " };
                if ui.button(format!("{}Status Bar", m)).clicked() {
                    self.status_bar = !self.status_bar;
                    ui.close_menu();
                }

                if self.is_markdown() {
                    ui.separator();
                    let preview_check = if self.preview_layout.is_visible() {
                        "✓ "
                    } else {
                        "  "
                    };
                    if ui.button(format!("{}Preview", preview_check)).clicked() {
                        if self.preview_layout.is_visible() {
                            self.preview_layout = PreviewLayout::Hidden;
                        } else {
                            self.preview_layout = PreviewLayout::Tabs;
                            self.active_tab = EditorTab::Editor;
                            self.editor_focus_request = true;
                        }
                        ui.close_menu();
                    }

                    if self.preview_layout.is_visible() {
                        ui.menu_button("Layout", |ui| {
                            for layout in [PreviewLayout::Tabs, PreviewLayout::Horizontal] {
                                let check = if self.preview_layout == layout {
                                    "✓ "
                                } else {
                                    "  "
                                };
                                if ui.button(format!("{}{}", check, layout.name())).clicked() {
                                    self.preview_layout = layout;
                                    ui.close_menu();
                                }
                            }
                        });
                        if ui.button(format!("Switch Tab\t{}+P", key)).clicked() {
                            self.toggle_preview_tab();
                            ui.close_menu();
                        }
                    }
                }

                ui.separator();
                ui.menu_button("Theme", |ui| {
                    for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
                        let check = if self.theme_mode == mode {
                            "✓ "
                        } else {
                            "  "
                        };
                        if ui.button(format!("{}{}", check, mode.name())).clicked() {
                            self.theme_mode = mode;
                            self.apply_theme(ui.ctx());
                            ui.close_menu();
                        }
                    }
                });
            });

            ui.menu_button("Language", |ui| {
                let auto_check = if self.auto_detect { "✓ " } else { "  " };
                let current_display = self.current_syntax.as_deref().unwrap_or("Plain Text");
                if ui
                    .button(format!("{}Auto ({})", auto_check, current_display))
                    .clicked()
                {
                    self.set_auto_mode();
                    self.update_preview_state();
                    ui.close_menu();
                }
                ui.separator();

                let all_langs = self.all_languages.clone();
                let current_clone = self.current_syntax.clone();
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for lang in &all_langs {
                            let check = if Some(lang.as_str()) == current_clone.as_deref()
                                && !self.auto_detect
                            {
                                "✓ "
                            } else {
                                "  "
                            };
                            if ui.button(format!("{}{}", check, lang)).clicked() {
                                self.set_manual_language(lang);
                                self.update_preview_state();
                                ui.close_menu();
                            }
                        }
                    });
            });

            ui.menu_button("Format", |ui| {
                let m = if self.word_wrap { "✓ " } else { "  " };
                if ui.button(format!("{}Word Wrap", m)).clicked() {
                    self.word_wrap = !self.word_wrap;
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About MemoChan").clicked() {
                    self.about = true;
                    ui.close_menu();
                }
            });
        });
    }

    fn main_area(&mut self, ctx: &egui::Context, dark: bool) {
        match self.preview_layout {
            PreviewLayout::Hidden => {
                egui::CentralPanel::default().show(ctx, |ui| self.editor(ui, dark));
            }
            PreviewLayout::Tabs => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let editor_response =
                            ui.selectable_label(self.active_tab == EditorTab::Editor, "Editor");
                        let preview_response =
                            ui.selectable_label(self.active_tab == EditorTab::Preview, "Preview");

                        if editor_response.clicked() {
                            self.active_tab = EditorTab::Editor;
                            self.editor_focus_request = true;
                        }
                        if preview_response.clicked() {
                            self.active_tab = EditorTab::Preview;
                            self.preview_focus_request = true;
                        }
                    });
                    ui.separator();
                    match self.active_tab {
                        EditorTab::Editor => self.editor(ui, dark),
                        EditorTab::Preview => self.preview(ui, dark),
                    }
                });
            }
            PreviewLayout::Horizontal => {
                egui::SidePanel::left("editor_panel")
                    .default_width(ctx.screen_rect().width() / 2.0)
                    .show(ctx, |ui| self.editor(ui, dark));
                egui::CentralPanel::default().show(ctx, |ui| self.preview(ui, dark));
            }
        }
    }

    fn editor(&mut self, ui: &mut egui::Ui, dark: bool) {
        let text_color = if dark {
            egui::Color32::from_rgb(0xd4, 0xd4, 0xd4)
        } else {
            egui::Color32::from_rgb(0x33, 0x33, 0x33)
        };

        let avail_rect = ui.available_rect_before_wrap();
        let response = ui.interact(avail_rect, egui::Id::new("ed_bg"), egui::Sense::click());
        if response.secondary_clicked() {
            self.context_menu = true;
        }
        if response.clicked() {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("ed")));
        }
        if self.first_frame {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("ed")));
            self.first_frame = false;
        }
        if self.editor_focus_request {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("ed")));
            self.editor_focus_request = false;

            // Scroll to cursor position when gaining focus
            if let Some(state) =
                egui::widgets::text_edit::TextEditState::load(ui.ctx(), egui::Id::new("ed"))
            {
                if let Some(cursor_range) = state.cursor.char_range() {
                    let cursor_pos = cursor_range.primary.index;
                    let line_count = self.text[..cursor_pos.min(self.text.len())]
                        .matches('\n')
                        .count();
                    let line_height = self.font_size * 1.5;
                    self.editor_scroll_offset = (line_count as f32 * line_height).max(0.0);
                }
            }
        }

        let f = FontId::proportional(self.font_size);
        let f_for_layouter = f.clone();
        let wrap = self.word_wrap;
        let syntax_name = self.current_syntax.clone();
        let highlighter = &self.highlighter;

        let mut scroll = if self.word_wrap {
            egui::ScrollArea::vertical()
        } else {
            egui::ScrollArea::both()
        };
        let mut changed = false;
        scroll = scroll.vertical_scroll_offset(self.editor_scroll_offset);

        let scroll_output = scroll
            .id_salt("ed")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut layouter = |ui: &egui::Ui, txt: &str, w: f32| {
                    let mut job = if let Some(ref name) = syntax_name {
                        if let Some(syntax) = highlighter.find_syntax_by_name(name) {
                            highlighter.highlight(txt, syntax, dark, f_for_layouter.size)
                        } else {
                            let mut j = egui::text::LayoutJob::default();
                            j.append(
                                txt,
                                0.0,
                                egui::TextFormat::simple(f_for_layouter.clone(), text_color),
                            );
                            j
                        }
                    } else {
                        let mut j = egui::text::LayoutJob::default();
                        j.append(
                            txt,
                            0.0,
                            egui::TextFormat::simple(f_for_layouter.clone(), text_color),
                        );
                        j
                    };
                    if wrap {
                        job.wrap.max_width = w;
                    } else {
                        job.wrap.max_width = f32::INFINITY;
                        job.wrap.break_anywhere = false;
                    }
                    ui.fonts(|f| f.layout_job(job))
                };

                let te = egui::TextEdit::multiline(&mut self.text)
                    .id(egui::Id::new("ed"))
                    .font(f)
                    .desired_width(f32::INFINITY)
                    .desired_rows(1)
                    .lock_focus(true)
                    .layouter(&mut layouter);

                let o = te.show(ui);
                changed = o.response.changed();

                if let Some(cursor_pos) = self.search_cursor_pos.take() {
                    let mut state = o.state.clone();
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(cursor_pos),
                        )));
                    state.store(ui.ctx(), egui::Id::new("ed"));
                }

                o
            });

        if !self.syncing_scroll {
            let new_offset = scroll_output.state.offset.y;
            if (new_offset - self.editor_scroll_offset).abs() > 1.0 {
                self.editor_scroll_offset = new_offset;
            }
        }
        self.syncing_scroll = false;
        self.editor_scroll_max = scroll_output.inner_rect.height();
        self.upd_cur(&scroll_output.inner);
        if changed {
            self.save_undo();
            self.modified = true;
        }
    }

    fn preview(&mut self, ui: &mut egui::Ui, _dark: bool) {
        let preview_id = egui::Id::new("preview_scroll");

        if self.preview_focus_request {
            ui.memory_mut(|m| m.request_focus(preview_id));
            self.preview_focus_request = false;
        }

        let output = egui::ScrollArea::vertical()
            .id_salt(preview_id)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                egui_commonmark::CommonMarkViewer::new().show(
                    ui,
                    &mut self.markdown_cache,
                    &self.text,
                );
            });

        self.preview_scroll_max = output.inner_rect.height();
        self.preview_scroll_offset = output.state.offset.y;
    }

    fn status(&self, ui: &mut egui::Ui, _dark: bool) {
        ui.horizontal(|ui| {
            let lang = self.current_syntax.as_deref().unwrap_or("Plain Text");
            let mode = if self.auto_detect { "Auto" } else { "Manual" };
            ui.label(format!("Ln {}, Col {}", self.cur_line, self.cur_col));
            ui.separator();
            ui.label(format!("{} [{}]", lang, mode));
            if self.is_markdown() && self.preview_layout.is_visible() {
                ui.separator();
                ui.label(format!("Preview: {}", self.preview_layout.name()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("UTF-8");
                ui.add_space(8.0);
                ui.label(format!("{}%", self.zoom));
            });
        });
    }

    fn show_context_menu(&mut self, ctx: &egui::Context) {
        let mut close_menu = false;
        let mut activate_search = false;
        let mut toggle_tab = false;

        egui::Area::new(egui::Id::new("ctx_menu"))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    if ui.button("Undo").clicked() {
                        self.undo();
                        close_menu = true;
                    }
                    if ui.button("Redo").clicked() {
                        self.redo();
                        close_menu = true;
                    }
                    ui.separator();
                    if ui.button("Cut").clicked() {
                        close_menu = true;
                    }
                    if ui.button("Copy").clicked() {
                        close_menu = true;
                    }
                    if ui.button("Paste").clicked() {
                        close_menu = true;
                    }
                    ui.separator();
                    if ui.button("Select All").clicked() {
                        close_menu = true;
                    }
                    ui.separator();
                    if ui.button("Find...").clicked() {
                        activate_search = true;
                        close_menu = true;
                    }
                    if self.preview_layout.is_visible() {
                        ui.separator();
                        if ui.button("Switch Tab").clicked() {
                            toggle_tab = true;
                            close_menu = true;
                        }
                    }
                    if ui.button("Insert Time/Date").clicked() {
                        self.ins_time();
                        close_menu = true;
                    }
                });
            });

        ctx.input(|i| {
            if i.pointer.any_click() && !close_menu {
                close_menu = true;
            }
        });

        if close_menu {
            if activate_search {
                self.search_active = true;
                self.search_focus = true;
                self.perform_search();
            }
            if toggle_tab {
                self.toggle_preview_tab();
            }
            self.context_menu = false;
        }
    }

    fn about_md(&mut self, ctx: &egui::Context) {
        let mut close = false;
        egui::Area::new(egui::Id::new("abg"))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                ui.allocate_rect(ctx.screen_rect(), egui::Sense::click());
                ui.painter().rect_filled(
                    ctx.screen_rect(),
                    0.0,
                    egui::Color32::from_black_alpha(180),
                );
            });
        egui::Area::new(egui::Id::new("adlg"))
            .order(egui::Order::Tooltip)
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(300.0);
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical_centered(|ui| {
                                if let Some(ref t) = self.icon {
                                    let s = t.size_vec2();
                                    let sc = 56.0 / s.x.max(s.y).max(1.0);
                                    ui.add(egui::Image::new((t.id(), s * sc)));
                                }
                                ui.add_space(8.0);
                                ui.heading(RichText::new("MemoChan").strong().size(18.0));
                                ui.label(
                                    RichText::new(env!("CARGO_PKG_VERSION")).size(11.0).weak(),
                                );
                            });
                        });
                        ui.add_space(10.0);
                        ui.label(RichText::new("Code Editor with Syntax Highlighting").size(12.0));
                        ui.label(
                            RichText::new(format!("Author: {}", env!("CARGO_PKG_AUTHORS")))
                                .size(10.0)
                                .weak(),
                        );
                        ui.label(
                            RichText::new(format!("License: {}", env!("CARGO_PKG_LICENSE")))
                                .size(10.0)
                                .weak(),
                        );
                        ui.add_space(10.0);
                        if ui.button("Close").clicked() {
                            close = true;
                        }
                    });
                });
            });
        if close {
            self.about = false;
        }
    }

    fn conf_md(&mut self, ctx: &egui::Context) {
        let mut sv = false;
        let mut ns = false;
        let mut ca = false;
        egui::Area::new(egui::Id::new("cbg"))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                ui.allocate_rect(ctx.screen_rect(), egui::Sense::click());
                ui.painter().rect_filled(
                    ctx.screen_rect(),
                    0.0,
                    egui::Color32::from_black_alpha(180),
                );
            });
        egui::Area::new(egui::Id::new("cdlg"))
            .order(egui::Order::Tooltip)
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(280.0);
                    ui.label(RichText::new("Save changes?").size(13.0));
                    ui.label(
                        RichText::new("[S] Save  [D] Don't Save  [Esc] Cancel")
                            .size(10.0)
                            .weak(),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save [S]").clicked() {
                            sv = true;
                        }
                        if ui.button("Don't Save [D]").clicked() {
                            ns = true;
                        }
                        if ui.button("Cancel [Esc]").clicked() {
                            ca = true;
                        }
                    });
                });
            });
        if sv {
            self.conf_save();
        }
        if ns {
            self.conf_nosave();
        }
        if ca {
            self.confirm = false;
            self.confirm_act = ConfirmAct::None;
        }
    }

    fn large_file_confirm_md(&mut self, ctx: &egui::Context) {
        let mut open = false;
        let mut cancel = false;

        egui::Area::new(egui::Id::new("lfbg"))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                ui.allocate_rect(ctx.screen_rect(), egui::Sense::click());
                ui.painter().rect_filled(
                    ctx.screen_rect(),
                    0.0,
                    egui::Color32::from_black_alpha(180),
                );
            });
        egui::Area::new(egui::Id::new("lfdlg"))
            .order(egui::Order::Tooltip)
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(320.0);
                    ui.label(RichText::new("Large File Warning").size(14.0).strong());
                    ui.add_space(8.0);
                    ui.label(RichText::new("This file is larger than 500KB.").size(12.0));
                    ui.label(
                        RichText::new("Opening large files may affect performance.")
                            .size(11.0)
                            .weak(),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("[O] Open Anyway  [Esc] Cancel")
                            .size(10.0)
                            .weak(),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Open [O]").clicked() {
                            open = true;
                        }
                        if ui.button("Cancel [Esc]").clicked() {
                            cancel = true;
                        }
                    });
                });
            });

        ctx.input(|i| {
            for e in &i.raw.events {
                if let egui::Event::Key {
                    key, pressed: true, ..
                } = e
                {
                    match key {
                        egui::Key::O => open = true,
                        egui::Key::Escape => cancel = true,
                        _ => {}
                    }
                }
            }
        });

        if open {
            if let Some(path) = self.pending_large_file.take() {
                if let Ok(c) = std::fs::read_to_string(&path) {
                    self.load_file_content(c, path);
                }
            }
            self.large_file_confirm = false;
        }
        if cancel {
            self.pending_large_file = None;
            self.large_file_confirm = false;
        }
    }
}
