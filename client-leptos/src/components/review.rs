use crate::{
    components::attachments::view_attachments,
    constants::{
        CLASS_HTML_PREVIEW, CLASS_ML_ACTION, CLASS_ML_BODY, CLASS_ML_BODY_FORM, CLASS_ML_CONTAINER,
        CLASS_ML_EDITOR, CLASS_ML_TOOLS, CLASS_MSG_ADDRESS, CLASS_MSG_ROW, CLASS_SCROLLED,
        MESSAGE_PREF,
    },
    store::{APP_EDITOR, APP_MESSAGES, APP_REVIEW, APP_USER, APP_VIEW, AppEditor, AppView},
    transport::send_data_conn,
    utils::{md_html, modal_header, svg_icon_height},
};
use leptos::{ev, html::*, prelude::*, task::spawn_local};
use shared::{DataConnParams, DataEBox, DataFileParam, DataMailAddress, DataMessageParams};

pub fn view_review() -> impl IntoView {
    || APP_REVIEW.get().map(review)
}

fn review(idb: i64) -> impl IntoView {
    div().class(CLASS_ML_CONTAINER).child(
        div()
            .class(CLASS_ML_EDITOR)
            .class(|| ("modal-review", true))
            .child((
                modal_header("Почтовое сообщение", review_close),
                review_actions(),
                div()
                    .class(CLASS_ML_BODY)
                    .class(|| (CLASS_SCROLLED, true))
                    .child(div().class(CLASS_ML_BODY_FORM).child(message_review(idb))),
            )),
    )
}

fn review_close() {
    APP_REVIEW.set(None);
}

pub fn message_review(idb: i64) -> impl IntoView {
    move || {
        APP_MESSAGES.read().ok().and_then(|map| {
            map.get(&idb).map(|data| {
                (
                    div().class(CLASS_MSG_ROW).child((
                        span().child("От кого:"),
                        span()
                            .class(CLASS_MSG_ADDRESS)
                            .child(view_address(&data.sender)),
                    )),
                    div().class(CLASS_MSG_ROW).child((
                        span().child("Кому:"),
                        span()
                            .class(CLASS_MSG_ADDRESS)
                            .child(view_address(&data.recipient)),
                    )),
                    div()
                        .class(CLASS_MSG_ROW)
                        .child((span().child("Тема:"), span().child(data.subject.clone()))),
                    data.attachments.clone().map(|attach| {
                        if attach.list.is_empty() {
                            ().into_any()
                        } else {
                            view_attachments(&attach).into_any()
                        }
                    }),
                    div()
                        .class("message-body")
                        .class(|| (CLASS_HTML_PREVIEW, true))
                        .inner_html(md_html(&data.content)),
                )
            })
        })
    }
}

fn view_address(address: &DataMailAddress) -> String {
    if let Some(name) = address.name.clone()
        && !name.is_empty()
    {
        format!("{} {name}", address.address)
    } else {
        address.address.clone()
    }
}

fn review_actions() -> impl IntoView {
    let idb = APP_REVIEW.get_untracked().unwrap_or_default();
    APP_MESSAGES.read().ok().and_then(|map| {
        map.get(&idb).map(|data| {
            div().class(CLASS_ML_TOOLS).child(ACTIONS.iter().fold(
                Vec::new(),
                |mut acc, (title, svg, action)| {
                    let flag = match *action {
                        ReviewAction::Reply | ReviewAction::Forward | ReviewAction::ToSaved => true,
                        ReviewAction::Download => data.has_source,
                    };
                    if flag {
                        acc.push(
                            button()
                                .class(CLASS_ML_ACTION)
                                .title(*title)
                                .child(svg_icon_height(svg, 24))
                                .on(ev::click, move |_| {
                                    spawn_local(async move { action_result(*action) })
                                }),
                        );
                    }

                    acc
                },
            ))
        })
    })
}

fn action_result(action: ReviewAction) {
    let idb = APP_REVIEW.get_untracked().unwrap_or_default();
    let current_box = match APP_VIEW.get_untracked() {
        AppView::Incoming => Some(DataEBox::Incoming),
        AppView::Sent => Some(DataEBox::Sent),
        AppView::Deleted => Some(DataEBox::Deleted),
        AppView::Saved => Some(DataEBox::Saved),
        _ => None,
    };
    let Some(current_box) = current_box else {
        return;
    };

    review_close();

    match action {
        ReviewAction::Reply => action_reply(idb),
        ReviewAction::Forward => action_forward(idb),
        ReviewAction::ToSaved => {
            send_data_conn(DataConnParams::Message(DataMessageParams::Move(
                idb,
                current_box,
                DataEBox::Saved,
            )));
        }
        ReviewAction::Download => {
            send_data_conn(DataConnParams::Files(DataFileParam::Letter(idb)));
        }
    }
}

fn action_reply(idb: i64) {
    APP_MESSAGES.read().ok().and_then(|map| {
        map.get(&idb).map(|data| {
            let reply_pref = "\n > ";
            let content = format!(
                "{MESSAGE_PREF}\n\n---{}\n\n{reply_pref}{}",
                APP_USER.get_untracked().signature,
                data.content.replace('\n', reply_pref)
            );
            APP_EDITOR.set(Some(AppEditor {
                recipient: RwSignal::new(data.sender.to_string()),
                content: RwSignal::new(content),
                subject: RwSignal::new(format!("RE: {}", data.subject)),
                ..AppEditor::default()
            }));
        })
    });
}

fn action_forward(idb: i64) {
    APP_MESSAGES.read().ok().and_then(|map| {
        map.get(&idb).map(|data| {
            let sender = data.sender.to_string();
            let content = format!(
                "**Отправитель:** {sender}\n**Переадресовано с:** {}\n\n---\n\n{}",
                APP_USER.get_untracked().email,
                data.content
            );
            APP_EDITOR.set(Some(AppEditor {
                content: RwSignal::new(content),
                subject: RwSignal::new(format!("FW: {}", data.subject)),
                ..AppEditor::default()
            }));
            if data.attachments.clone().map(|data| !data.list.is_empty()) == Some(true) {
                send_data_conn(DataConnParams::Files(DataFileParam::Forward(idb)));
            }
        })
    });
}

#[derive(Clone, Copy)]
enum ReviewAction {
    Reply,
    Forward,
    Download,
    ToSaved,
}

const ACTIONS: [(&str, &str, ReviewAction); 4] = [
    ("ответить", REPLY_SVG, ReviewAction::Reply),
    ("переслать", FORWARD_SVG, ReviewAction::Forward),
    (
        "переместить в сохраненные",
        TO_SAVED_SVG,
        ReviewAction::ToSaved,
    ),
    ("скачать оригинал", DOWNLOAD_SVG, ReviewAction::Download),
];

const REPLY_SVG: &str = include_str!("../../../assets/icons/review/reply.svg");
const FORWARD_SVG: &str = include_str!("../../../assets/icons/review/share.svg");
const TO_SAVED_SVG: &str = include_str!("../../../assets/icons/review/envelope-open-text.svg");
const DOWNLOAD_SVG: &str = include_str!("../../../assets/icons/review/floppy-disk.svg");
