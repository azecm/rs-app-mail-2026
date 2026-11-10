mod content;
mod list;
mod modals;
mod tools;
mod utils;

use self::content::view_notes_content;
use self::list::view_notes_list;
use self::modals::notes_modal_view;
use self::tools::view_notes_tools;
use self::utils::{CurrentIdn, ReadNotesMap, use_notes_map};
use crate::constants::{CLASS_CONTENT_WRAPPER, CLASS_HIDDEN};
use crate::store::{APP_VIEW, AppView};
use leptos::html::*;
use leptos::prelude::*;

pub use self::utils::{update_email_list, update_events};

pub fn view_notes() -> impl IntoView {
    let not_notes = move || APP_VIEW.get() != AppView::Notes;
    provide_context::<CurrentIdn>(RwSignal::new(0));
    provide_context::<ReadNotesMap>(use_notes_map());

    div()
        .class(CLASS_CONTENT_WRAPPER)
        .class(move || (CLASS_HIDDEN, not_notes()))
        .child((view_layout(), notes_modal_view()))
}

fn view_layout() -> impl IntoView {
    (
        div()
            .class("notes-left")
            .child((view_notes_tools(), view_notes_list())),
        div().class("notes-right").child(view_notes_content()),
    )
}
