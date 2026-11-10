use crate::{
    constants::{CHAR_LF, Cmd},
    normalize::NormalizeContent,
    text::text_job,
};
use egui::{
    TextBuffer,
    text::{CCursor, CCursorRange},
    text_edit::TextEditOutput,
};
use wasm_bindgen::prelude::*;
use web_time::{SystemTime, UNIX_EPOCH};

// Define the external JavaScript interface
// #[wasm_bindgen(module = "/js/external.js")]
#[wasm_bindgen(module = "/dist/md-editor.mjs")]
extern "C" {
    fn editor_open(content: &str);
    fn editor_close() -> String;
}

#[derive(Default, Debug)]
pub struct MdEditor {
    content: String,
    selected: String,
    debug_str: String,
    //temp_content_2: String,
    cursor_index: Option<usize>,
    cmd: Option<Cmd>,
    click_time: f64,
}

impl MdEditor {
    pub fn new(source: &str, text: &str) -> Self {
        Self {
            content: NormalizeContent::new(source).read(),
            selected: text.to_lowercase(),
            ..Self::default()
        }
    }
    pub fn set_selected(&mut self, text: &str) {
        self.selected = text.to_lowercase();
    }
    pub fn get_selected(&mut self) -> String {
        self.selected.clone()
    }
    pub fn get_content(&self) -> String {
        self.content.clone()
    }
    pub fn view(&mut self, _ui: &mut egui::Ui) {
        editor_open("# www\n\n444");
    }
    pub fn view_(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("debug: {:?}", self.debug_str));

        let cmd_remove = self.get_cmd_remove();
        let mut delta = 0;
        let mut cmd: Option<Cmd> = None;
        let mut content_updated = String::new();
        //let mut content_temp_1 = String::new();
        //let mut content_temp_2 = String::new();
        let source_len = self.content.len();
        let mut layout = |ui: &egui::Ui, buf: &dyn TextBuffer, wrap_width: f32| {
            let next_len = buf.as_str().len();
            let mut job = if next_len != source_len {
                if source_len > next_len {
                    cmd = cmd_remove;
                } else if let Some(index) = self.cursor_index
                    && let Some(char) = buf.as_str().chars().nth(index)
                {
                    cmd = Some(Cmd::Add(index, char));
                }
                let mut norm = NormalizeContent::new(buf.as_str());
                //content_temp_1 = buf.as_str().to_string();
                delta = norm.delta(cmd);
                //content_temp_2 = norm.content.clone();

                content_updated = norm.read();

                text_job(&content_updated, &self.selected)
            } else {
                text_job(buf.as_str(), &self.selected)
            };

            job.wrap.max_width = wrap_width;
            ui.fonts_mut(|f| f.layout_job(job))
        };

        let editor = egui::TextEdit::multiline(&mut self.content)
            .font(egui::TextStyle::Monospace) // for cursor height
            .code_editor()
            .desired_rows(10)
            .lock_focus(true)
            .desired_width(f32::INFINITY)
            .layouter(&mut layout)
            .show(ui);

        if let Some(index) = get_click_index(ui, &editor) {
            let current = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|f| f.as_secs_f64())
                .unwrap_or_default();
            let is_second = current - self.click_time < 0.5;
            //log::info!("Clicked at {} {is_second}", index);
            self.click_time = current;
            match get_clicked(index - 1, &self.content) {
                ClickData::None => {}
                ClickData::Url(url) => {
                    if is_second {
                        let _ = webbrowser::open(url.as_str()).is_ok();
                    }
                }
                ClickData::Replace(index, char) => {
                    self.content = self
                        .content
                        .chars()
                        .enumerate()
                        .map(|(ind, c)| if ind == index { char } else { c })
                        .collect();
                }
            }
        }

        self.cursor_position(ui, editor, self.content.chars().count());

        if cmd.is_some() {
            self.cmd = cmd;
        }
        if !content_updated.is_empty() {
            self.content = content_updated;
        }
    }

    fn get_cmd_remove(&self) -> Option<Cmd> {
        if let Some(ind) = self.cursor_index
            && ind > 0
            && let Some(c) = self.content.chars().nth(ind - 1)
        {
            Some(Cmd::Remove(ind - 1, c))
        } else {
            None
        }
    }

    fn cursor_position(
        &mut self,
        ui: &mut egui::Ui,
        mut editor: TextEditOutput,
        max_position: usize,
    ) {
        if editor.response.lost_focus() || editor.response.has_focus() {
            let space_pressed = ui.input(|i| i.key_pressed(egui::Key::Space));

            self.debug_str = ui.input(|i| {
                format!(
                    "{:?} {}",
                    i.pointer.delta(),
                    //i.pointer.direction(),
                    i.pointer.is_moving()
                )
            });
            if space_pressed {
                /*self.debug_str = ui.input(|i| {
                    format!(
                        "{:?} {:?} {}",
                        i.pointer.delta(),
                        i.pointer.direction(),
                        i.pointer.is_moving()
                    )
                });*/
            } else {
                let ind_prev = self.cursor_index.unwrap_or_default();
                let mut cursor_index = get_cursor_index(&editor);
                let ind_cur = cursor_index.unwrap_or_default();

                let mut need_update = false;
                let mut char_prev: char = 0.into();
                self.content.chars().enumerate().for_each(|(ind, char)| {
                    if ind_cur == ind && char_prev == char && char == CHAR_LF {
                        need_update = true;
                    }
                    char_prev = char;
                });
                if need_update {
                    let delta: i64 = if ind_prev > ind_cur { -1 } else { 1 };
                    let next = (ind_cur as i64 + delta) as usize;
                    let next = next.clamp(0, max_position);
                    let min = CCursor::new(next);
                    let max = CCursor::new(next);
                    let new_range = CCursorRange::two(min, max);
                    cursor_index = Some(next);
                    editor.state.cursor.set_char_range(Some(new_range));
                    editor.state.store(ui.ctx(), editor.response.id);
                }

                self.cursor_index = cursor_index;
            }
        }
    }
}

fn get_click_index(ui: &egui::Ui, editor: &TextEditOutput) -> Option<usize> {
    if let Some(Some(pointer_pos)) =
        ui.input(|i| i.pointer.any_click().then_some(i.pointer.latest_pos()))
        && editor.response.rect.contains(pointer_pos)
    {
        let local_pos = pointer_pos - editor.response.rect.left_top();
        let char_index = editor.galley.cursor_from_pos(local_pos).index;
        Some(char_index)
    } else {
        None
    }
}

#[derive(Debug)]
enum ClickData {
    None,
    Url(String),
    Replace(usize, char),
}

fn get_clicked(char_index: usize, content: &str) -> ClickData {
    let mut result = ClickData::None;

    let mut current_box = String::new();
    let mut current_url = String::new();
    let mut flag_url = false;
    let mut flag_checkbox = false;
    content.chars().enumerate().for_each(|(i, c)| {
        if char_index == i {
            flag_url = true;
            if !current_box.is_empty() {
                flag_checkbox = true;
            }
            // log::info!("CLICK index: {char_index} '{c}'");
        }
        // --
        let prev = current_box.len();
        match prev {
            0 if c == '[' => {
                current_box.push(c);
            }
            1 if (c == 'x' || c == 'X' || c == ' ') => {
                current_box.push(c);
            }
            2 if c == ']' => {
                current_box.push(c);
            }
            _ => {}
        }
        if prev == current_box.len() && prev > 0 {
            if flag_checkbox {
                match current_box.to_ascii_lowercase().as_str() {
                    "[x]" => {
                        result = ClickData::Replace(i - 2, ' ');
                    }
                    "[ ]" => {
                        result = ClickData::Replace(i - 2, 'X');
                    }
                    _ => {}
                }
            }
            flag_checkbox = false;
            current_box = String::new();
        }
        // --
        if !(c.is_alphabetic()
            || c.is_ascii_digit()
            || c == '.'
            || c == '?'
            || c == '_'
            || c == '='
            || c == '&'
            || c == '-'
            || c == ':'
            || c == '/')
        {
            if flag_url {
                if current_url.contains("://") {
                    result = ClickData::Url(current_url.clone());
                }
                flag_url = false;
            }
            current_url = String::new();
        } else {
            current_url.push(c);
        }
    });

    result
}

fn get_cursor_index(editor: &TextEditOutput) -> Option<usize> {
    if let Some(c) = editor.cursor_range
        && c.primary.index == c.secondary.index
    {
        Some(c.primary.index)
    } else {
        None
    }
}
