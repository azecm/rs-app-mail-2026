use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/static/js/md-editor.mjs")]
extern "C" {
    fn editor_start(content: &str);
    fn editor_result() -> String;
    fn editor_read() -> String;
    fn editor_close();
}

pub fn codemirror_start(content: &str) {
    let parser = pulldown_cmark::Parser::new(content);
    let parser = parser.map(|event| match event {
        pulldown_cmark::Event::HardBreak => pulldown_cmark::Event::SoftBreak,
        _ => event,
    });
    let mut output_markdown = String::new();
    pulldown_cmark_to_cmark::cmark(parser, &mut output_markdown).unwrap_or_default();

    editor_start(&output_markdown);
}

pub fn codemirror_read() -> String {
    prepare(editor_read())
}

pub fn codemirror_close() {
    editor_close();
}

pub fn codemirror_result() -> String {
    prepare(editor_result())
}

fn prepare(source: String) -> String {
    let parser = pulldown_cmark::Parser::new(&source);
    let parser = parser.map(|event| match event {
        pulldown_cmark::Event::SoftBreak => pulldown_cmark::Event::HardBreak,
        _ => event,
    });
    let mut output_markdown = String::new();
    pulldown_cmark_to_cmark::cmark(parser, &mut output_markdown).unwrap_or_default();

    output_markdown
}
