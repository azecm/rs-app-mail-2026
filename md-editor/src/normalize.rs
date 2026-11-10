use crate::constants::{CHAR_EMPTY_LINE, CHAR_LF, CHAR_PAREN, CHAR_PERIOD, CHAR_SPACE, Cmd};
use comrak::arena_tree::NodeEdge;
use comrak::nodes::{ListDelimType, ListType, NodeLink, NodeList, NodeTaskItem, NodeValue};
use comrak::{Arena, Options, parse_document};

#[derive(Default)]
pub struct NormalizeContent {
    started: bool,
    pub content: String,
    header_space: bool,
    list_item: Option<NodeList>,
    list_task: Option<NodeTaskItem>,
    list: Option<NodeList>,
    block_quote: usize,
}

impl NormalizeContent {
    pub fn new(source: &str) -> Self {
        Self {
            content: if source.is_empty() { ".\n\n" } else { source }.to_string(),
            ..Self::default()
        }
    }
    pub fn delta(&mut self, cmd: Option<Cmd>) -> i32 {
        let mut delta = 0;
        let mut lf_counter = 0;
        let mut space_counter = 0;

        if let Some(Cmd::Remove(ind, CHAR_LF)) = cmd {
            for i in [ind - 1, ind] {
                if self.content.chars().nth(i) == Some(CHAR_LF)
                    && let Some((byte_idx, _)) = self.content.char_indices().nth(i)
                {
                    self.content.remove(byte_idx);
                }
            }
        }

        let mut symbol_counter: usize = 0;
        let ff = |mut acc: String, ch: char| -> String {
            if ch != CHAR_EMPTY_LINE {
                let cur_space_counter = space_counter;
                if ch == CHAR_SPACE {
                    space_counter += 1;
                } else {
                    space_counter = 0;
                }

                if ch == CHAR_LF {
                    // пробел в конце строки
                    if cur_space_counter == 1 {
                        acc.push(CHAR_EMPTY_LINE);
                    }
                    if symbol_counter == 1 {
                        acc.push(CHAR_EMPTY_LINE);
                    }
                    symbol_counter = 0;
                } else {
                    symbol_counter += 1;
                }

                // исключаем перенос строки \s\s\n
                if ch == CHAR_LF && cur_space_counter < 2 {
                    lf_counter += 1;
                } else {
                    // новая строка
                    if lf_counter % 2 == 1 {
                        lf_counter += 1;
                    }
                    let all = lf_counter / 2;
                    for ind in 0..all {
                        acc.push(CHAR_LF);
                        acc.push(CHAR_LF);
                        if ind < all - 1 {
                            acc.push(CHAR_EMPTY_LINE);
                        }
                    }
                    // --

                    acc.push(ch);
                    lf_counter = 0;
                }
            }
            acc
        };

        self.content = self.content.chars().fold(String::new(), ff);

        for _ in 0..lf_counter {
            self.content.push(CHAR_LF);
        }
        if self.content.ends_with("\n\n") {
            self.content.push(CHAR_EMPTY_LINE);
            if let Some(Cmd::Add(_, _)) = cmd {
                delta = 1;
            }
        }

        delta
    }

    pub fn read(&mut self) -> String {
        let arena = Arena::new();
        let mut options = Options::default();
        options.extension.tasklist = true;
        let root = parse_document(&arena, &self.content, &options);

        self.content = String::new();

        // display_tree(root);

        for node in root.traverse() {
            match node {
                NodeEdge::Start(node) => match &node.data.borrow().value {
                    NodeValue::Document => {}
                    NodeValue::Paragraph => {
                        self.paragraph_start();
                    }
                    NodeValue::Heading(heading) => {
                        self.header_start(heading.level as usize);
                    }
                    NodeValue::Text(text) => {
                        self.add_text(text);
                    }
                    NodeValue::SoftBreak => {
                        //self.add_soft_break();
                    }
                    NodeValue::LineBreak => {
                        self.add_line_break();
                    }
                    NodeValue::ThematicBreak => {
                        self.add_thematic_break();
                    }
                    NodeValue::Emph => {
                        self.italic_start();
                    }
                    NodeValue::Strong => {
                        self.bold_start();
                    }
                    NodeValue::Link(_) => {
                        self.link_start();
                    }
                    NodeValue::List(node) => {
                        self.list_start(node);
                    }
                    NodeValue::Item(node) => {
                        self.list_item(node);
                    }
                    NodeValue::TaskItem(node) => {
                        self.list_task(node);
                    }
                    NodeValue::BlockQuote => {
                        self.block_quote_start();
                    }
                    _ => {
                        log::info!("?? {:?}", &node.data.borrow().value);
                    }
                },
                NodeEdge::End(node) => match &node.data.borrow().value {
                    NodeValue::Paragraph => {
                        self.paragraph_end();
                    }
                    NodeValue::Heading(_) => {
                        self.header_end();
                    }
                    NodeValue::Emph => {
                        self.italic_end();
                    }
                    NodeValue::Strong => {
                        self.bold_end();
                    }
                    NodeValue::Link(node) => {
                        self.link_end(node);
                    }
                    NodeValue::List(_) => {
                        self.list_end();
                    }
                    NodeValue::BlockQuote => {
                        self.block_quote_end();
                    }
                    _ => {}
                },
            }
        }

        self.content.clone()
    }

    fn paragraph_start(&mut self) {
        if self.started {
            self.content.push(CHAR_LF);
        }
        self.started = true;
        self.block_quote_view();

        if let Some(list) = self.list
            && let Some(task) = self.list_task.take()
        {
            self.content.push(list.bullet_char as char);
            self.content.push(' ');
            self.content.push('[');
            self.content.push(task.symbol.unwrap_or(' '));
            self.content.push(']');
            self.content.push(' ');
        }

        if let Some(node) = self.list_item.take() {
            match node.list_type {
                ListType::Bullet => {
                    self.content.push(node.bullet_char as char);
                    self.content.push(' ');
                }
                ListType::Ordered => {
                    let delim = match node.delimiter {
                        ListDelimType::Period => CHAR_PERIOD,
                        ListDelimType::Paren => CHAR_PAREN,
                    };
                    self.content.push_str(node.start.to_string().as_str());
                    self.content.push(delim);
                    self.content.push(' ');
                }
            }
        }
    }
    fn paragraph_end(&mut self) {
        self.content.push(CHAR_LF);
    }
    fn header_start(&mut self, level: usize) {
        if self.started {
            self.content.push(CHAR_LF);
        }
        self.started = true;
        self.block_quote_view();
        for _ in 0..level {
            self.content.push('#');
        }
        self.header_space = true;
        self.started = true;
    }
    fn header_end(&mut self) {
        self.content.push(CHAR_LF);
        self.header_space = false;
    }

    //fn add_soft_break(&mut self) {
    //    self.content.push(CHAR_LF);
    //}

    fn add_line_break(&mut self) {
        self.content.push_str("  \n");
    }

    fn add_thematic_break(&mut self) {
        self.content.push_str("\n---\n");
    }

    fn add_text(&mut self, text: &str) {
        if self.header_space {
            self.content.push(CHAR_SPACE);
        }
        self.content.push_str(text);
    }
    fn list_start(&mut self, node: &NodeList) {
        self.list = Some(*node);
    }
    fn list_end(&mut self) {
        self.list = None;
    }
    fn list_item(&mut self, node: &NodeList) {
        self.list_item = Some(*node);
    }
    fn list_task(&mut self, node: &NodeTaskItem) {
        self.list_task = Some(*node);
    }
    fn link_start(&mut self) {
        self.content.push('[');
    }
    fn link_end(&mut self, link: &NodeLink) {
        self.content.push(']');
        self.content.push('(');
        self.content.push_str(link.url.as_str());
        self.content.push(')');
    }

    fn italic_start(&mut self) {
        self.content.push('*');
    }
    fn italic_end(&mut self) {
        self.content.push('*');
    }
    fn bold_start(&mut self) {
        self.content.push_str("**");
    }
    fn bold_end(&mut self) {
        self.content.push_str("**");
    }

    fn block_quote_view(&mut self) {
        if self.block_quote > 0 {
            self.content.push_str(&">".repeat(self.block_quote));
            self.content.push(' ');
        }
    }
    fn block_quote_start(&mut self) {
        self.block_quote += 1;
    }
    fn block_quote_end(&mut self) {
        self.block_quote -= 1;
    }
}

fn display_child<'a>(node: comrak::Node<'a>, level: usize, out: &mut String) {
    out.push('\n');
    for n in node.children() {
        let l = "-".repeat(level * 2);
        out.push_str(format!("{l} {:?}", n.data.borrow()).as_str());
        out.push('\n');
        if n.first_child().is_some() {
            display_child(n, level + 1, out);
        }
    }
}

#[allow(unused)]
fn display_tree<'a>(node: comrak::Node<'a>) {
    let mut out = String::new();
    display_child(node, 1, &mut out);
    log::info!("{out}");
}
