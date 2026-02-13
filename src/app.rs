use crate::config::{
    calculate_font_size, APP_ICON, DEFAULT_FONT_SIZE, DEFAULT_ZOOM_LEVEL, MAX_ZOOM_LEVEL,
    MIN_ZOOM_LEVEL,
};
use chrono::Local;
use eframe::egui;
use egui::{ColorImage, FontId, RichText, TextureHandle};
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

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
}

#[derive(Clone, Debug)]
enum FdAct {
    Open,
    SaveAs(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ConfirmAct {
    None,
    New,
    Open,
    Exit,
}

impl Default for Notepad {
    fn default() -> Self {
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
        }
    }
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

impl Notepad {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        if let Ok(img) = image::load_from_memory(APP_ICON) {
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
        app
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
                        if let Ok(c) = std::fs::read_to_string(&p) {
                            self.text = c;
                            self.last_saved = self.text.clone();
                            self.undo_stack.clear();
                            self.redo_stack.clear();
                            self.file = Some(p);
                            self.modified = false;
                        }
                    }
                    Some(FdAct::SaveAs(c)) => {
                        if std::fs::write(&p, c).is_ok() {
                            self.file = Some(p);
                            self.modified = false;
                        }
                    }
                    None => {}
                }
                self.fd_rx = None;
                self.fd_act = None;
            }
        }
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
            let _ = tx.send(FileDialog::new().add_filter("Text", &["txt"]).pick_file());
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

    fn save_as(&mut self) {
        let c = self.text.clone();
        let (tx, rx) = mpsc::channel();
        self.fd_rx = Some(rx);
        self.fd_act = Some(FdAct::SaveAs(c));
        std::thread::spawn(move || {
            let _ = tx.send(FileDialog::new().add_filter("Text", &["txt"]).save_file());
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

        let mut zi = false;
        let mut zo = false;
        let mut zr = false;

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
                                _ => {}
                            }
                        }
                        if modifiers.ctrl && !modifiers.mac_cmd {
                            match key {
                                egui::Key::Plus | egui::Key::Equals => zi = true,
                                egui::Key::Minus => zo = true,
                                egui::Key::Num0 => zr = true,
                                _ => {}
                            }
                        }
                        match key {
                            egui::Key::F5 => self.ins_time(),
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
                        match key {
                            egui::Key::Escape | egui::Key::Enter => self.about = false,
                            _ => {}
                        }
                    }
                }
            });
        }

        if self.confirm {
            ctx.input(|i| {
                for e in &i.raw.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = e
                    {
                        if *key == egui::Key::Escape {
                            self.confirm = false;
                            self.confirm_act = ConfirmAct::None;
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

        egui::TopBottomPanel::top("menu").show(ctx, |ui| self.menu(ui));
        if self.status_bar {
            egui::TopBottomPanel::bottom("status").show(ctx, |ui| self.status(ui));
        }
        egui::CentralPanel::default().show(ctx, |ui| self.editor(ui));

        if self.about {
            self.about_md(ctx);
        }
        if self.confirm {
            self.conf_md(ctx);
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
            ui.menu_button("Format", |ui| {
                let m = if self.word_wrap { "✓ " } else { "  " };
                if ui.button(format!("{}Word Wrap", m)).clicked() {
                    self.word_wrap = !self.word_wrap;
                    ui.close_menu();
                }
            });
            ui.menu_button("View", |ui| {
                if ui.button(format!("Zoom In\t{}++", key)).clicked() {
                    self.z_in();
                    ui.close_menu();
                }
                if ui.button(format!("Zoom Out\t{}+-", key)).clicked() {
                    self.z_out();
                    ui.close_menu();
                }
                if ui.button(format!("Reset Zoom\t{}+0", key)).clicked() {
                    self.z_res();
                    ui.close_menu();
                }
                ui.separator();
                let m = if self.status_bar { "✓ " } else { "  " };
                if ui.button(format!("{}Status Bar", m)).clicked() {
                    self.status_bar = !self.status_bar;
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

    fn editor(&mut self, ui: &mut egui::Ui) {
        let f = FontId::proportional(self.font_size);
        let f2 = f.clone();
        let wrap = self.word_wrap;

        let mut layouter = move |ui: &egui::Ui, txt: &str, w: f32| {
            let mut job = egui::text::LayoutJob::default();
            if wrap {
                job.wrap.max_width = w;
            } else {
                job.wrap.max_width = f32::INFINITY;
                job.wrap.break_anywhere = false;
            }
            job.append(
                txt,
                0.0,
                egui::TextFormat {
                    font_id: f2.clone(),
                    ..Default::default()
                },
            );
            ui.fonts(|f| f.layout_job(job))
        };

        // Use interact instead of allocate_rect to handle click-to-focus without affecting layout
        let avail_rect = ui.available_rect_before_wrap();
        let response = ui.interact(avail_rect, egui::Id::new("ed_bg"), egui::Sense::click());
        if response.clicked() {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("ed")));
        }

        if self.first_frame {
            ui.memory_mut(|m| m.request_focus(egui::Id::new("ed")));
            self.first_frame = false;
        }

        if self.word_wrap {
            egui::ScrollArea::vertical()
                .id_salt("ed")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let te = egui::TextEdit::multiline(&mut self.text)
                        .id(egui::Id::new("ed"))
                        .font(f)
                        .desired_width(f32::INFINITY)
                        .desired_rows(1)
                        .lock_focus(true)
                        .layouter(&mut layouter);
                    let o = te.show(ui);
                    self.upd_cur(&o);
                    if o.response.changed() {
                        self.save_undo();
                        self.modified = true;
                    }
                });
        } else {
            egui::ScrollArea::both()
                .id_salt("ed")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let te = egui::TextEdit::multiline(&mut self.text)
                        .id(egui::Id::new("ed"))
                        .font(f)
                        .desired_width(f32::INFINITY)
                        .desired_rows(1)
                        .lock_focus(true)
                        .layouter(&mut layouter);
                    let o = te.show(ui);
                    self.upd_cur(&o);
                    if o.response.changed() {
                        self.save_undo();
                        self.modified = true;
                    }
                });
        }
    }

    fn status(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Ln {}, Col {}", self.cur_line, self.cur_col));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("UTF-8");
                ui.add_space(8.0);
                ui.label(format!("{}%", self.zoom));
            });
        });
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
                    ui.set_max_width(300.0);
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        if let Some(ref t) = self.icon {
                            let s = t.size_vec2();
                            let sc = 56.0 / s.x.max(s.y).max(1.0);
                            ui.add(egui::Image::new((t.id(), s * sc)));
                        }
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            ui.heading(RichText::new("MemoChan").strong().size(18.0));
                            ui.label(RichText::new(env!("CARGO_PKG_VERSION")).size(11.0).weak());
                        });
                    });
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Simple notepad with egui").size(12.0));
                        ui.add_space(4.0);
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
                        ui.add_space(14.0);
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
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            sv = true;
                        }
                        if ui.button("Don't Save").clicked() {
                            ns = true;
                        }
                        if ui.button("Cancel").clicked() {
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
}
