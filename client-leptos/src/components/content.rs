use super::messages::view_messages;
use super::notes::view_notes;
use leptos::html::*;
use leptos::prelude::*;

pub fn view_content() -> impl IntoView {
    div()
        .class("content-container")
        .child((view_messages(), view_notes()))
}
