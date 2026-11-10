use leptos::html::*;
use leptos::prelude::*;

pub fn app_hello() -> AnyView {
    div()
        .child("Здравия желаю!")
        .class("block-hello")
        .into_any()
}
