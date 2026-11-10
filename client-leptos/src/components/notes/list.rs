use super::utils::{CurrentIdn, ReadNotesMap, block_hidden, row_by_idn};
use crate::constants::{CLASS_SCROLLED, CLASS_SELECTED};
use crate::dom::query_selector_all;
use crate::store::{APP_NOTE_FOCUS, APP_NOTES_EDITING};
use leptos::ev;
use leptos::html::*;
use leptos::prelude::*;

pub fn view_notes_list() -> impl IntoView {
    let notes_map = use_context::<ReadNotesMap>().expect("context::notes_map");
    let current_id = use_context::<CurrentIdn>().expect("context::CurrentIdn");
    Effect::new(move || {
        let next = APP_NOTE_FOCUS.get();
        if next > 0
            && let Some(row) = row_by_idn(next)
        {
            for elem in query_selector_all("details").into_iter() {
                elem.remove_attribute("open").ok();
                if let Some(idp) = elem
                    .get_attribute("data-idp")
                    .and_then(|str| str.parse::<i32>().ok())
                    && Some(idp) == row.idp
                {
                    elem.set_attribute("open", "").ok();
                }
            }
            current_id.set(next);
        }
    });
    div()
        .class("notes-list-container")
        .class(|| (CLASS_SCROLLED, true))
        .class(|| block_hidden(true))
        .child(move || {
            get_parents(notes_map)
                .iter()
                .map(|r| view_parent_row(notes_map, r.clone()))
                .collect_view()
        })
}

fn set_current(signal: CurrentIdn, idn: i32) {
    APP_NOTES_EDITING.set(None);
    signal.set(idn);
}

fn view_parent_row(map: ReadNotesMap, (idn, label): (i32, String)) -> impl IntoView {
    let current = use_context::<CurrentIdn>().unwrap_or_default();
    details().attr("data-idp", idn).child((
        summary()
            .child(label)
            .class(move || (CLASS_SELECTED, current.get() == idn))
            .on(ev::click, move |_| set_current(current, idn)),
        ul().class("notes-items").child(move || {
            get_children(map, idn)
                .iter()
                .map(|r| view_child_row(r.clone()))
                .collect_view()
        }),
    ))
}

fn view_child_row((idn, label, email): (i32, String, String)) -> impl IntoView {
    let current = use_context::<CurrentIdn>().unwrap_or_default();
    li().class("notes-item").child(
        button()
            .class(move || (CLASS_SELECTED, current.get() == idn))
            .child((label, span().child(email)))
            .on(ev::click, move |_| set_current(current, idn)),
    )
}

fn get_parents(map: ReadNotesMap) -> Vec<(i32, String)> {
    let mut results = Vec::new();
    if let Some(rows) = map.get().get(&0) {
        for idn in rows {
            results.push((
                *idn,
                row_by_idn(*idn)
                    .unwrap_or_default()
                    .label
                    .unwrap_or_default(),
            ));
        }
    }
    results
}

fn get_children(map: ReadNotesMap, parent_idn: i32) -> Vec<(i32, String, String)> {
    let mut results = Vec::new();
    if let Some(rows) = map.get().get(&parent_idn) {
        for idn in rows {
            let row = row_by_idn(*idn).unwrap_or_default();
            let label = row.label.unwrap_or_default();
            let email = row.email.unwrap_or_default();
            let label = if label.trim().is_empty() {
                "---".to_string()
            } else {
                label
            };
            let email = if email.trim().is_empty() {
                email
            } else {
                format!("[{email}]")
            };
            results.push((*idn, label, email));
        }
    }
    results
}
