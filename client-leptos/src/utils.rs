use crate::constants::{CLASS_ML_ACTION, CLASS_ML_HEADER, MESSAGE_PREF};
use crate::dom::{local_storage_remove, storage_key};
use crate::store::{APP_EDITOR, APP_MODAL, APP_STATE, APP_USER, AppEditor, AppModal, AppState};
use leptos::prelude::*;
use leptos::svg::{path, svg};
use leptos::{ev, html::*};

pub fn svg_icon(data: &str) -> impl IntoView {
    svg_icon_height(data, 20)
}

pub fn svg_icon_height(data: &str, h: usize) -> impl IntoView {
    let (view_box, path_data) = svg_data(data);
    svg()
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("viewBox", view_box)
        .attr("height", h)
        .child(path().attr("d", path_data).attr("fill", "currentColor"))
}

pub fn icon_xmark() -> impl IntoView {
    svg_icon_height(SVG_XMARK, 24)
}

pub fn modal_header(title: &str, close: fn()) -> impl IntoView {
    div().class(CLASS_ML_HEADER).child((
        div().child(title),
        button()
            .class(CLASS_ML_ACTION)
            .title("закрыть")
            .child(icon_xmark())
            .on(ev::click, move |_| close()),
    ))
}

fn svg_data(data: &str) -> (String, String) {
    let mut path = String::new();
    let mut view_box = String::new();
    let mut attr = String::new();
    let mut state: usize = 0;
    data.chars().for_each(|ch| {
        match state {
            0 if ch.is_whitespace() => {
                state = 1;
                attr = String::new();
            }
            1 => {
                if ch == '"' {
                    state = 2;
                } else if ch.is_whitespace() {
                    attr = String::new();
                } else if ch.is_alphabetic() {
                    attr.push(ch);
                }
            }
            2 => {
                if ch == '"' {
                    state = 0;
                } else {
                    match attr.as_str() {
                        "d" => {
                            path.push(ch);
                        }
                        "viewBox" => {
                            view_box.push(ch);
                        }
                        _ => {}
                    };
                }
            }
            _ => {}
        };
    });
    (view_box, path)
}

pub fn app_modal(title: &str, content: &str, confirm: Option<fn()>) {
    APP_MODAL.set(Some(AppModal {
        title: title.to_string(),
        content: content.to_string(),
        confirm,
    }));
}

pub fn app_modal_error(content: &str) {
    app_modal("Ошибка", content, None);
}

/*
pub fn app_modal_test(text: &str) {
    let show_msg = APP_USER.get_untracked().email == "_";
    let mut prev = APP_MODAL
        .get_untracked()
        .map(|d| d.content)
        .unwrap_or_default();

    if show_msg {
        if text.is_empty() {
            prev = String::new();
        }
        prev.push(' ');
        prev += text;
        app_modal("Тест", &prev, None);
    }
}
*/

pub fn logout() {
    local_storage_remove(&storage_key());
    APP_STATE.set(AppState::Login(String::new()));
}

pub fn mail_to(recipient: &str, subject: &str, content: &str) {
    let content = if content.is_empty() {
        MESSAGE_PREF.to_string()
    } else {
        content.to_string()
    };
    let content = format!("{content}\n\n---{}\n", APP_USER.get_untracked().signature);
    APP_EDITOR.set(Some(AppEditor {
        recipient: RwSignal::new(recipient.to_string()),
        content: RwSignal::new(content),
        subject: RwSignal::new(subject.to_string()),
        ..AppEditor::default()
    }));
}

pub fn md_html(source: &str) -> String {
    let parser = pulldown_cmark::Parser::new(source);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

const SVG_XMARK: &str = include_str!("../../assets/icons/xmark.svg");
