use crate::constants::{CHAR_PAREN, CHAR_PERIOD, CHAR_SPACE};
use crate::fonts::{font_bold, font_bold_italic, font_italic};
use comrak::nodes::{ListDelimType, ListType, NodeLink, NodeList, NodeTaskItem};
use eframe::epaint::Stroke;
use egui::text::LayoutJob;
use egui::{Color32, FontFamily, FontId};

use comrak::arena_tree::NodeEdge;
use comrak::nodes::NodeValue;
use comrak::{Arena, Options, parse_document};

pub fn text_job(source: &str, selected: &str) -> LayoutJob {
    let mut job = TextJob::default();

    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.tasklist = true;
    let root = parse_document(&arena, source, &options);

    for node in root.traverse() {
        match node {
            NodeEdge::Start(node) => match &node.data.borrow().value {
                NodeValue::Document => {}
                NodeValue::Paragraph => {
                    job.paragraph_start();
                }
                NodeValue::Heading(heading) => {
                    job.header_start(heading.level as usize);
                }
                NodeValue::Text(text) => {
                    job.add_text(text, selected);
                }
                NodeValue::SoftBreak => {
                    job.add_soft_break();
                }
                NodeValue::LineBreak => {
                    job.add_line_break();
                }
                NodeValue::ThematicBreak => {
                    job.add_thematic_break();
                }
                NodeValue::Emph => {
                    job.italic_start();
                }
                NodeValue::Strong => {
                    job.bold_start();
                }
                NodeValue::Link(_) => {
                    job.link_start();
                }
                NodeValue::List(node) => {
                    job.list_start(node);
                }
                NodeValue::Item(node) => {
                    job.list_item(node);
                }
                NodeValue::TaskItem(node) => {
                    job.list_task(node);
                }
                NodeValue::BlockQuote => {
                    job.block_quote_start();
                }
                _ => {
                    log::info!("?? {:?}", &node.data.borrow().value);
                }
            },
            NodeEdge::End(node) => match &node.data.borrow().value {
                NodeValue::Heading(_) => {
                    job.header_end();
                }
                NodeValue::Emph => {
                    job.italic_end();
                }
                NodeValue::Strong => {
                    job.bold_end();
                }
                NodeValue::Link(node) => {
                    job.link_end(node, selected);
                }
                NodeValue::List(_) => {
                    job.list_end();
                }
                NodeValue::BlockQuote => {
                    job.block_quote_end();
                }
                _ => {}
            },
        }
    }

    job.job
}

#[derive(Default)]
struct TextJob {
    pub job: LayoutJob,
    link: bool,
    bold: bool,
    italic: bool,
    header: usize,
    header_space: bool,
    block_quote: usize,

    started: bool,
    list_item: Option<NodeList>,
    list: Option<NodeList>,
    list_task: Option<NodeTaskItem>, //double_break: bool,
}

const COLOR_TEXT: Color32 = Color32::GRAY;

const COLOR_HEADER: Color32 = Color32::BROWN;
const COLOR_MARK: Color32 = Color32::MAGENTA;
const COLOR_ITALIC: Color32 = Color32::PURPLE;
const COLOR_BOLD: Color32 = Color32::ORANGE;
const COLOR_LINK: Color32 = Color32::LIGHT_BLUE;
const COLOR_HL_BG: Color32 = Color32::MAGENTA;
const COLOR_HL_TX: Color32 = Color32::WHITE;

const MARK_BOLD: &str = "**";
const MARK_HEADER: &str = "#";
const MARK_HEADER_AFTER: &str = " ";
const MARK_ITALIC: &str = "*";
const MARK_THEMATIC_BREAK: &str = "---";

impl TextJob {
    pub fn paragraph_start(&mut self) {
        if self.started {
            self.add_break();
            //if self.double_break {
            self.add_break();
            //}
            //self.double_break = true;
        }
        self.block_quote_view();

        if let Some(list) = self.list
            && let Some(task) = self.list_task.take()
        {
            self.mark(&String::from_utf8(vec![list.bullet_char]).unwrap_or_default());
            self.add_space();
            self.mark(&format!("[{}]", task.symbol.unwrap_or(' ')));
            self.add_space();
        }

        if let Some(node) = self.list_item.take() {
            match node.list_type {
                ListType::Bullet => {
                    self.mark(&String::from_utf8(vec![node.bullet_char]).unwrap_or_default());
                    self.add_space();
                }
                ListType::Ordered => {
                    let delim = match node.delimiter {
                        ListDelimType::Period => CHAR_PERIOD,
                        ListDelimType::Paren => CHAR_PAREN,
                    };
                    self.mark(&format!("{}{delim}", node.start));
                    self.add_space();
                }
            }
        }
    }

    pub fn header_start(&mut self, level: usize) {
        self.header = level;
        self.header_space = true;
        if self.started {
            self.add_break();
            self.add_break();
        }
        self.block_quote_view();
        self.job.append(
            MARK_HEADER.repeat(self.header).as_str(),
            0.0,
            egui::TextFormat {
                font_id: self.font_default(),
                color: COLOR_MARK,
                ..Default::default()
            },
        );
        self.started = true;
        //self.double_break = false;
    }

    pub fn header_end(&mut self) {
        self.header = 0;
    }

    pub fn bold_start(&mut self) {
        self.bold = true;
        self.mark(MARK_BOLD);
    }

    pub fn italic_start(&mut self) {
        self.italic = true;
        self.mark(MARK_ITALIC);
    }

    pub fn bold_end(&mut self) {
        self.mark(MARK_BOLD);
        self.bold = false;
    }

    pub fn italic_end(&mut self) {
        self.mark(MARK_ITALIC);
        self.italic = false;
    }

    pub fn add_break(&mut self) {
        self.job.append(
            "\n",
            0.0,
            egui::TextFormat {
                ..Default::default()
            },
        );
    }

    pub fn add_soft_break(&mut self) {
        self.job.append(
            CHAR_SPACE.to_string().as_str(),
            0.0,
            egui::TextFormat {
                ..Default::default()
            },
        );
    }

    pub fn add_line_break(&mut self) {
        self.job.append(
            format!("{CHAR_SPACE}{CHAR_SPACE}\n").as_str(),
            0.0,
            egui::TextFormat {
                ..Default::default()
            },
        );
    }

    pub fn add_thematic_break(&mut self) {
        // https://github.github.com/gfm/#thematic-breaks
        self.add_break();
        self.add_break();
        self.mark(MARK_THEMATIC_BREAK);
    }

    fn mark(&mut self, text: &str) {
        self.job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: self.font_default(),
                color: COLOR_MARK,
                ..Default::default()
            },
        );
    }

    fn font_default(&self) -> FontId {
        FontId::new(
            if self.header > 0 {
                28.0 - (self.header as f32) * 2.0
            } else {
                14.0
            },
            FontFamily::Proportional,
        )
    }

    fn font_current(&self) -> FontId {
        let mut ff = FontFamily::Proportional;
        if self.bold && self.italic {
            ff = font_bold_italic();
        } else if self.bold {
            ff = font_bold();
        } else if self.italic {
            ff = font_italic();
        }
        FontId::new(
            if self.header > 0 {
                28.0 - (self.header as f32) * 2.0
            } else {
                14.0
            },
            ff,
        )
    }
    fn add_space(&mut self) {
        self.job.append(
            MARK_HEADER_AFTER,
            0.0,
            egui::TextFormat {
                font_id: self.font_default(),
                ..Default::default()
            },
        );
    }

    fn text_selected(&mut self, text: &str, phrase: &str, text_format: egui::TextFormat) {
        let mut text = text;
        let phrase_count: usize = phrase.chars().count() + 1;

        while !text.is_empty() {
            let mut find = false;
            let mut last = 0;
            let mut pos: Vec<usize> = Vec::new();
            pos.push(0);
            for ch in text.chars() {
                let current = ch.to_string().len();
                last += current;
                if pos.len() < phrase_count {
                    pos.push(last);
                } else {
                    for i in 1..phrase_count {
                        pos[i - 1] = pos[i];
                    }
                    pos[phrase_count - 1] = last;
                }

                if pos.len() == phrase_count
                    && text[pos[0]..pos[phrase_count - 1]].to_lowercase() == phrase
                {
                    find = true;
                    if pos[0] > 0 {
                        self.job.append(&text[..pos[0]], 0.0, text_format.clone());
                    }
                    self.job.append(
                        &text[pos[0]..pos[phrase_count - 1]],
                        0.0,
                        egui::TextFormat {
                            //font_id: FontId::proportional(20.0),
                            color: COLOR_HL_TX,
                            background: COLOR_HL_BG,
                            ..Default::default()
                        },
                    );
                    text = &text[pos[phrase_count - 1]..];
                    break;
                }
            }
            if !find {
                self.job.append(text, 0.0, text_format.clone());
                text = "";
            }
        }
    }

    pub fn add_text(&mut self, text: &str, selected: &str) {
        if self.header > 0 && self.header_space {
            self.header_space = false;
            self.add_space();
        }

        let mut color = COLOR_TEXT;
        if self.header > 0 {
            color = COLOR_HEADER;
        }
        if self.italic && self.bold {
            color = Color32::BLUE;
        } else if self.bold {
            color = COLOR_BOLD;
        } else if self.italic {
            color = COLOR_ITALIC;
        } else if self.link {
            color = COLOR_LINK;
        }

        if !selected.is_empty() && text.to_lowercase().contains(selected) {
            self.text_selected(
                text,
                selected,
                egui::TextFormat {
                    color,
                    ..Default::default()
                },
            );
        } else {
            self.job.append(
                text,
                0.0,
                egui::TextFormat {
                    font_id: self.font_current(),
                    color,
                    ..Default::default()
                },
            );
        }

        self.started = true;
    }
    pub fn list_start(&mut self, node: &NodeList) {
        self.list = Some(*node);
    }
    pub fn list_end(&mut self) {
        self.list = None;
    }
    pub fn list_item(&mut self, node: &NodeList) {
        self.list_item = Some(*node);
    }
    pub fn list_task(&mut self, node: &NodeTaskItem) {
        self.list_task = Some(*node);
    }
    pub fn link_start(&mut self) {
        self.mark("[");
        self.link = true;
    }

    pub fn link_end(&mut self, link: &NodeLink, selected: &str) {
        self.link = false;
        self.mark("]");
        self.mark("(");

        let text_format = egui::TextFormat {
            font_id: self.font_current(),
            underline: Stroke {
                width: 1.,
                color: COLOR_LINK,
            },
            color: COLOR_LINK,
            ..Default::default()
        };

        if !selected.is_empty() && link.url.to_lowercase().contains(selected) {
            self.text_selected(link.url.as_str(), selected, text_format);
        } else {
            self.job.append(link.url.as_str(), 0.0, text_format);
        }
        self.mark(")");
    }
    // --
    fn block_quote_view(&mut self) {
        if self.block_quote > 0 {
            self.mark(&">".repeat(self.block_quote));
            self.add_space();
        }
    }
    pub fn block_quote_start(&mut self) {
        self.block_quote += 1;
    }
    pub fn block_quote_end(&mut self) {
        self.block_quote -= 1;
    }
}
