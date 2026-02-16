use egui::text::LayoutJob;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Theme as SyntectTheme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn get_dark_theme(&self) -> &SyntectTheme {
        &self.theme_set.themes["base16-mocha.dark"]
    }

    pub fn get_light_theme(&self) -> &SyntectTheme {
        &self.theme_set.themes["base16-ocean.light"]
    }

    pub fn highlight(
        &self,
        text: &str,
        syntax: &SyntaxReference,
        dark: bool,
        font_size: f32,
    ) -> LayoutJob {
        let theme = if dark {
            self.get_dark_theme()
        } else {
            self.get_light_theme()
        };

        let mut h = HighlightLines::new(syntax, theme);
        let mut job = LayoutJob::default();

        for line in LinesWithEndings::from(text) {
            let ranges = h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            for (style, txt) in ranges {
                let color = style.foreground;
                let egui_color = egui::Color32::from_rgb(color.r, color.g, color.b);

                let mut format =
                    egui::TextFormat::simple(egui::FontId::proportional(font_size), egui_color);
                if style.font_style.contains(FontStyle::BOLD) {
                    format.font_id.size = font_size * 1.2;
                }
                if style.font_style.contains(FontStyle::ITALIC) {
                    // italics not well supported in egui
                }
                if style.font_style.contains(FontStyle::UNDERLINE) {
                    format.underline = egui::Stroke::new(1.0, egui_color);
                }

                job.append(txt, 0.0, format);
            }
        }

        job
    }

    #[allow(dead_code)]
    pub fn syntax_set(&self) -> &SyntaxSet {
        &self.syntax_set
    }

    pub fn find_syntax_by_extension(&self, ext: &str) -> Option<&SyntaxReference> {
        self.syntax_set.find_syntax_by_extension(ext)
    }

    pub fn find_syntax_by_name(&self, name: &str) -> Option<&SyntaxReference> {
        self.syntax_set.find_syntax_by_name(name)
    }

    pub fn all_syntaxes(&self) -> &[SyntaxReference] {
        self.syntax_set.syntaxes()
    }
}
