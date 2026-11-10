use crate::components::notes::update_events;
use crate::constants::{
    CLASS_ML_ACTIONS, CLASS_ML_BODY, CLASS_ML_CONTAINER, CLASS_ML_HEADER, CLASS_ML_WINDOW,
};
use crate::store::{APP_NOTE_FOCUS, APP_SEARCH_OPENED, APP_VIEW, AppView, EventItemStruct};
use leptos::{ev, html::*, prelude::*};

pub fn view_search() -> impl IntoView {
    || {
        APP_SEARCH_OPENED.get().then_some(
            div()
                .class(CLASS_ML_CONTAINER)
                .child(
                    div().class(CLASS_ML_WINDOW).child((
                        div().class(CLASS_ML_HEADER).child("Поиск"),
                        div().class(CLASS_ML_BODY).child(view_events()),
                        div()
                            .class(CLASS_ML_ACTIONS)
                            .child(button().child("закрыть").on(ev::click, |_| {
                                APP_SEARCH_OPENED.set(false);
                            })),
                    )),
                )
                .into_any(),
        )
    }
}

fn view_events() -> impl IntoView {
    ul().child(|| {
        update_events()
            .iter()
            .map(|r| view_event(r.clone()))
            .collect_view()
    })
}

fn view_event(row: EventItemStruct) -> impl IntoView {
    let d = row.date.split('-').collect::<Vec<_>>();
    let idn = row.idn;
    li().child(
        button()
            .child(format!(
                "{}.{}.{} {}: {}",
                d[2], d[1], d[0], row.group, row.label
            ))
            .on(ev::click, move |_| {
                APP_SEARCH_OPENED.set(false);
                APP_VIEW.set(AppView::Notes);
                APP_NOTE_FOCUS.set(idn);
            }),
    )
}
