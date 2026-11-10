use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

const INTER_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Inter-Regular.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../assets/fonts/Inter-Bold.ttf");
const INTER_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Inter-Italic.ttf");
const INTER_BOLD_ITALIC: &[u8] = include_bytes!("../../assets/fonts/Inter-BoldItalic.ttf");

// JetBrains Mono font family (code/monospace)
const JETBRAINS_REGULAR: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
//const JETBRAINS_BOLD: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf");
//const JETBRAINS_ITALIC: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Italic.ttf");
//const JETBRAINS_BOLD_ITALIC: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-BoldItalic.ttf");

pub const FONT_INTER: &str = "Inter";
const FONT_INTER_BOLD: &str = "Inter-Bold";
pub const FONT_INTER_ITALIC: &str = "Inter-Italic";
pub const FONT_INTER_BOLD_ITALIC: &str = "Inter-BoldItalic";
pub const FONT_JETBRAINS: &str = "JetBrainsMono";
//pub const FONT_JETBRAINS_BOLD: &str = "JetBrainsMono-Bold";
//pub const FONT_JETBRAINS_ITALIC: &str = "JetBrainsMono-Italic";
//pub const FONT_JETBRAINS_BOLD_ITALIC: &str = "JetBrainsMono-BoldItalic";

pub fn font_bold() -> FontFamily {
    FontFamily::Name(FONT_INTER_BOLD.into())
}

pub fn font_italic() -> FontFamily {
    FontFamily::Name(FONT_INTER_ITALIC.into())
}
pub fn font_bold_italic() -> FontFamily {
    FontFamily::Name(FONT_INTER_BOLD_ITALIC.into())
}

pub fn replace_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    let font_list = [
        (FONT_INTER, INTER_REGULAR, FontFamily::Proportional),
        (FONT_JETBRAINS, JETBRAINS_REGULAR, FontFamily::Monospace),
        (FONT_INTER_BOLD, INTER_BOLD, font_bold()),
        (FONT_INTER_ITALIC, INTER_ITALIC, font_italic()),
        (
            FONT_INTER_BOLD_ITALIC,
            INTER_BOLD_ITALIC,
            font_bold_italic(),
        ),
    ];

    for (k, data, ff) in font_list.into_iter() {
        fonts
            .font_data
            .insert(k.to_owned(), Arc::new(FontData::from_static(data)));
        fonts
            .families
            .entry(ff)
            .or_default()
            .insert(0, k.to_owned());
    }

    ctx.set_fonts(fonts);
}
