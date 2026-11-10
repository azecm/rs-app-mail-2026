use super::utils::{CurrentIdn, row_by_idn};
use crate::components::codemirror::{codemirror_result, codemirror_start};
use crate::components::notes::utils::block_hidden;
use crate::constants::{CLASS_HTML_PREVIEW, CLASS_SCROLLED};
use crate::store::{APP_IS_MOBILE, APP_NOTE_VERSION, APP_NOTES_EDITING};
use crate::transport::send_data_conn;
use crate::utils::md_html;
use leptos::html::*;
use leptos::prelude::*;
use shared::{DataConnParams, DataNoteAction, DataNoteParams};

pub fn view_notes_content() -> impl IntoView {
    let current_idn = use_context::<CurrentIdn>().expect("context::CurrentIdn");
    Effect::new(move || {
        let idn = current_idn.get_untracked();
        match APP_NOTES_EDITING.get() {
            Some(true) => {
                if let Some(row) = row_by_idn(idn) {
                    codemirror_start(&row.content.unwrap_or_default());
                }
            }
            Some(false) => {
                let result = codemirror_result();
                send_data_conn(DataConnParams::Notes(DataNoteParams {
                    idn,
                    content: Some(result),
                    action: DataNoteAction::Update,
                    ..DataNoteParams::default()
                }));
            }
            None => {}
        }
    });

    move || {
        APP_NOTE_VERSION.get();
        if APP_NOTES_EDITING.get() == Some(true) {
            div()
                .id("editor")
                .class(|| {
                    (
                        if APP_IS_MOBILE.get() {
                            "notes-content-mb"
                        } else {
                            "notes-content-dt"
                        },
                        true,
                    )
                })
                .class(|| block_hidden(false))
                .class(|| (CLASS_SCROLLED, true))
                .into_any()
        } else if !APP_IS_MOBILE.get() {
            div()
                .class("notes-content-dt")
                .class(|| (CLASS_SCROLLED, true))
                .class(|| (CLASS_HTML_PREVIEW, true))
                .inner_html(move || {
                    let row = row_by_idn(current_idn.get());
                    let content = row.unwrap_or_default().content.unwrap_or_default();
                    md_html(&content)
                })
                .into_any()
        } else {
            ().into_any()
        }
    }
}
