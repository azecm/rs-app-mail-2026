mod constants;
mod editor;
mod fonts;
mod normalize;
mod text;

pub use editor::MdEditor;
pub use fonts::replace_fonts;

/*
impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        replace_fonts(&cc.egui_ctx);
        Self {
            editor: MdEditor::new(),
        }
    }
}
*/
