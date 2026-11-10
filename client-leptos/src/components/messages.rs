use crate::components::review::message_review;
use crate::constants::{
    CLASS_CONTENT_WRAPPER, CLASS_HIDDEN, CLASS_SCROLLED, LOADING_FINISHED, LOADING_NEED,
};
use crate::dom::html_element_from_target;
use crate::store::{
    APP_BOXES, APP_IS_MOBILE, APP_MESSAGES, APP_REVIEW, APP_USER, APP_VIEW, AppView, BoxData,
};
use crate::transport::send_data_conn;
use crate::utils::svg_icon;
use leptos::{ev, html::*, prelude::*};
use shared::{DataConnParams, DataEBox, DataMessageBoxed, DataMessageParams};
use web_sys::Event;

pub type CurrentIdb = RwSignal<i64>;

pub fn view_messages() -> impl IntoView {
    provide_context::<CurrentIdb>(RwSignal::new(0));
    let not_box = move |cur: AppView| APP_VIEW.get() != cur;
    (
        div()
            .class(CLASS_CONTENT_WRAPPER)
            .class(move || (CLASS_HIDDEN, not_box(AppView::Incoming)))
            .child(view_data(DataEBox::Incoming)),
        div()
            .class(CLASS_CONTENT_WRAPPER)
            .class(move || (CLASS_HIDDEN, not_box(AppView::Saved)))
            .child(view_data(DataEBox::Saved)),
        div()
            .class(CLASS_CONTENT_WRAPPER)
            .class(move || (CLASS_HIDDEN, not_box(AppView::Sent)))
            .child(view_data(DataEBox::Sent)),
        div()
            .class(CLASS_CONTENT_WRAPPER)
            .class(move || (CLASS_HIDDEN, not_box(AppView::Deleted)))
            .child(view_data(DataEBox::Deleted)),
    )
}

#[derive(Clone)]
struct BoxContext {
    pub scroll_top: RwSignal<i32>,
    pub scroll_height: RwSignal<i32>,
    pub scrolled: RwSignal<i32>,
}

fn view_data(ebox: DataEBox) -> AnyView {
    let scroll_top: RwSignal<i32> = RwSignal::new(0);
    let scroll_height: RwSignal<i32> = RwSignal::new(0);
    let scrolled: RwSignal<i32> = RwSignal::new(0);
    provide_context::<BoxContext>(BoxContext {
        scroll_top,
        scroll_height,
        scrolled,
    });
    if let Some(box_data) = APP_BOXES.get(&ebox) {
        let page = box_data.page;
        let loading = box_data.loading;
        let finished = box_data.finished;
        (
            div()
                .class("messages-list")
                .class(|| (CLASS_SCROLLED, true))
                .child(view_items(ebox, box_data))
                .on(ev::scroll, move |e| {
                    let (flag, top, height) = need_next(e);
                    scroll_top.set(top);
                    scroll_height.set(height);
                    if (scrolled.get_untracked() - top).abs() > 20 {
                        scrolled.set(top);
                    }
                    if flag
                        && !finished.get_untracked()
                        && loading.get_untracked() == LOADING_FINISHED
                    {
                        loading.set(LOADING_NEED);
                        page.set(page.get_untracked() + 1);
                    }
                }),
            div()
                .class("messages-content")
                .class(|| (CLASS_SCROLLED, true))
                .child(|| (!APP_IS_MOBILE.get()).then(view_message)),
        )
            .into_any()
    } else {
        ().into_any()
    }
}

fn need_next(e: Event) -> (bool, i32, i32) {
    html_element_from_target(e.target())
        .map(|element| {
            let top = element.scroll_top() as i32;
            let height = element.offset_height();
            let scroll_max = element.scroll_height() - height;
            let flag = scroll_max - top < 50;
            (flag, top, height)
        })
        .unwrap_or_default()
}

const MESSAGE_GAP: i32 = 12;
const MESSAGE_HEIGHT: i32 = 65 + MESSAGE_GAP;

fn view_items(ebox: DataEBox, box_data: &BoxData) -> impl IntoView {
    let box_ctx = use_context::<BoxContext>().expect("BoxContext");
    let scroll_top = box_ctx.scroll_top;
    let scroll_height = box_ctx.scroll_height;
    let scrolled = box_ctx.scrolled;
    let list = box_data.list;

    // height top [idb]
    let (params, set_params) = signal::<(i32, i32, Vec<i64>)>((0, 0, Vec::new()));

    Effect::new(move || {
        scrolled.get();
        let min = scroll_top.get_untracked();
        let max = scroll_height.get_untracked() + min;
        let mut started = false;
        let mut active = false;
        let items = list.get();
        let all = items.len() as i32;
        let result_height = all * MESSAGE_HEIGHT - MESSAGE_GAP;
        let mut result_top: i32 = 0;
        let result_items = APP_MESSAGES
            .read()
            .ok()
            .map(|map| {
                items
                    .iter()
                    .enumerate()
                    .fold(Vec::<i64>::new(), |mut acc, (index, idb)| {
                        let index = index as i32;
                        let current = (index + 1) * MESSAGE_HEIGHT;
                        if current > min && !started {
                            started = true;
                            active = true;
                            result_top = index * MESSAGE_HEIGHT;
                        }
                        if active && map.get(idb).is_some() {
                            acc.push(*idb);
                        }
                        if max > 0 && current > max && active {
                            active = false;
                        }
                        acc
                    })
            })
            .unwrap_or_default();
        let (prev_height, prev_top, prev_items) = params.get_untracked();
        if result_height != prev_height
            || result_top != prev_top
            || result_items.len() != prev_items.len()
        {
            set_params.set((result_height, result_top, result_items));
        }
    });

    move || {
        let (height, top, items) = params.get();
        APP_MESSAGES.read().ok().map(move |map| {
            div().style(format!("height: {height}px;")).child(
                div()
                    .class("messages-items")
                    .style(format!("transform: translateY({top}px);"))
                    .child(items.iter().fold(Vec::<AnyView>::new(), |mut acc, idb| {
                        if let Some(data) = map.get(idb) {
                            acc.push(view_item(ebox, data));
                        }
                        acc
                    })),
            )
        })
    }
}

fn view_item(ebox: DataEBox, data: &DataMessageBoxed) -> AnyView {
    let current_idb = use_context::<CurrentIdb>().expect("context::CurrentIdb");
    let idb = data.idb;
    let address = match ebox {
        DataEBox::Incoming => data.sender.address.as_str(),
        DataEBox::Saved => data.sender.address.as_str(),
        DataEBox::Sent => data.recipient.address.as_str(),
        DataEBox::Deleted => {
            if APP_USER.get_untracked().email == data.sender.address {
                data.recipient.address.as_str()
            } else {
                data.sender.address.as_str()
            }
        }
    };
    let (move_label, move_svg, move_target) = icon_move(
        ebox == DataEBox::Deleted,
        data.sender.address == APP_USER.get_untracked().email,
    );
    div()
        .class("message-container")
        .child((
            button()
                .class("message-action")
                .child(svg_icon(move_svg))
                .title(move_label)
                .on(ev::click, move |_| {
                    send_data_conn(DataConnParams::Message(DataMessageParams::Move(
                        idb,
                        ebox,
                        move_target,
                    )));
                }),
            button()
                .class("message-preview")
                .child((
                    div().class("message-date").child((
                        code().child(data.date.as_str()),
                        data.attachments.clone().map(|r| {
                            (!r.list.is_empty()).then(|| format!("файлов: {}", r.list.len()))
                        }),
                    )),
                    div().class("message-address").child(address),
                    div().class("message-subject").child(data.subject.as_str()),
                ))
                .on(ev::mouseenter, move |_| {
                    current_idb.set(idb);
                })
                .on(ev::click, move |_| {
                    APP_REVIEW.set(Some(idb));
                }),
        ))
        .into_any()
}

fn view_message() -> impl IntoView {
    let current_idb = use_context::<CurrentIdb>().expect("context::CurrentIdb");
    move || {
        let idb = current_idb.get();
        message_review(idb)
    }
}

fn icon_move(is_trash: bool, is_sender: bool) -> (&'static str, &'static str, DataEBox) {
    if is_trash {
        if is_sender {
            MOVE_BTNS[2]
        } else {
            MOVE_BTNS[1]
        }
    } else {
        MOVE_BTNS[0]
    }
}

const MOVE_BTNS: [(&str, &str, DataEBox); 3] = [
    ("в корзину", SVG_TRASH, DataEBox::Deleted),
    ("во входящие", SVG_INBOX, DataEBox::Incoming),
    ("в отправленные", SVG_OUTBOX, DataEBox::Sent),
];

const SVG_TRASH: &str = include_str!("../../../assets/icons/bar/trash.svg");
const SVG_INBOX: &str = include_str!("../../../assets/icons/bar/incoming.svg");
const SVG_OUTBOX: &str = include_str!("../../../assets/icons/bar/sent.svg");
