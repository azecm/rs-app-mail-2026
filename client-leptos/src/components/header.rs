use crate::components::notes::update_events;
use crate::store::{APP_SEARCH_OPENED, APP_VIEW, AppView};
use crate::utils::{app_modal, logout, mail_to, svg_icon};
use leptos::ev;
use leptos::html::*;
use leptos::prelude::*;
use web_sys::MouseEvent;

pub fn view_header() -> impl IntoView {
    div().class("header-container").child((
        btn(SVG_NEW, "написать", None, |_| {
            mail_to("", "", "");
        }),
        btn(
            SVG_INBOX,
            "входящие",
            Some(AppView::Incoming),
            move |_| {
                APP_VIEW.set(AppView::Incoming);
            },
        ),
        btn(
            SVG_SAVED,
            "сохраненные",
            Some(AppView::Saved),
            move |_| {
                APP_VIEW.set(AppView::Saved);
            },
        ),
        btn(
            SVG_SEND,
            "отправленные",
            Some(AppView::Sent),
            move |_| {
                APP_VIEW.set(AppView::Sent);
            },
        ),
        btn(
            SVG_TRASH,
            "корзина",
            Some(AppView::Deleted),
            move |_| {
                APP_VIEW.set(AppView::Deleted);
            },
        ),
        btn(
            SVG_NOTES,
            "заметки",
            Some(AppView::Notes),
            move |_| {
                APP_VIEW.set(AppView::Notes);
            },
        ),
        btn_search(SVG_SEARCH, "найти"),
        btn(SVG_EXIT, "выйти", None, |_| {
            app_modal(
                "Подтверждение",
                "Вы действительно хотите выйти?",
                Some(logout),
            )
        }),
    ))
}

fn btn<F>(icon: &str, label: &str, t: Option<AppView>, cb: F) -> impl IntoView
where
    F: FnMut(MouseEvent) + 'static,
{
    let selected = move || ("selected", Some(APP_VIEW.get()) == t);
    button()
        .class("header-btn")
        .class(selected)
        .title(label)
        .child((svg_icon(icon), span().child(label)))
        .on(ev::click, cb)
}

fn btn_search(icon: &str, label: &str) -> impl IntoView {
    let (label, _) = signal(label.to_string());
    button()
        .class("header-btn")
        .title(label)
        .child((
            svg_icon(icon),
            span().child(move || {
                let events_count = update_events().len();
                if events_count > 0 {
                    format!("[{events_count}]")
                } else {
                    label.get_untracked()
                }
            }),
        ))
        .on(ev::click, |_| {
            APP_SEARCH_OPENED.set(true);
        })
}

const SVG_SEARCH: &str = include_str!("../../../assets/icons/bar/magnifying-glass.svg");
const SVG_NEW: &str = include_str!("../../../assets/icons/bar/new.svg");
const SVG_INBOX: &str = include_str!("../../../assets/icons/bar/incoming.svg");
const SVG_SAVED: &str = include_str!("../../../assets/icons/bar/saved.svg");
const SVG_SEND: &str = include_str!("../../../assets/icons/bar/sent.svg");
const SVG_TRASH: &str = include_str!("../../../assets/icons/bar/trash.svg");
const SVG_NOTES: &str = include_str!("../../../assets/icons/bar/notes.svg");
const SVG_EXIT: &str = include_str!("../../../assets/icons/bar/exit.svg");
