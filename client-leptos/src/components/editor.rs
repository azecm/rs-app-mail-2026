use crate::components::attachments::editor_attachments;
use crate::components::codemirror::{codemirror_read, codemirror_start};
use crate::components::notes::update_email_list;
use crate::constants::{
    CLASS_CONFIRM, CLASS_HIDDEN, CLASS_HTML_PREVIEW, CLASS_ML_ACTIONS, CLASS_ML_BODY,
    CLASS_ML_BODY_FORM, CLASS_ML_CONTAINER, CLASS_ML_EDITOR,
};
use crate::dom::{file_to_vec, html_input_from_target};
use crate::store::{APP_EDITOR, AppEditor};
use crate::transport::send_data_conn;
use crate::utils::{app_modal_error, md_html, modal_header, svg_icon};
use leptos::task::spawn_local;
use leptos::{ev, html::*, prelude::*};
use shared::{DataConnParams, DataFileItem, DataFileParam, DataMessageParams, FileUpload};
use web_sys::Event;

pub fn view_editor() -> impl IntoView {
    || APP_EDITOR.get().map(editor)
}

fn editor(data: AppEditor) -> impl IntoView {
    codemirror_start(&data.content.get_untracked());
    let (preview, set_preview) = signal(false);
    let recipient = data.recipient;
    let subject = data.subject;
    let send_disabled = data.send_disabled;
    let upload_progress = data.upload_progress;
    let attachments = data.attachments;
    let list_id = "email-datalist";
    div()
        .class(CLASS_ML_CONTAINER)
        .child(
            div().class(CLASS_ML_EDITOR).child((
                datalist().id(list_id).child(|| {
                    update_email_list()
                        .iter()
                        .map(|row| option().child(row.clone()))
                        .collect_view()
                }),
                modal_header("Почтовое сообщение", editor_close),
                div().class(CLASS_ML_BODY).child(
                    div().class(CLASS_ML_BODY_FORM).child((
                        input()
                            .placeholder("получатель")
                            .value(recipient.get_untracked())
                            .list(list_id)
                            .on(ev::input, move |e| recipient.set(event_target_value(&e))),
                        input()
                            .placeholder("тема")
                            .value(subject.get_untracked())
                            .on(ev::input, move |e| subject.set(event_target_value(&e))),
                        editor_attachments(attachments),
                        div()
                            .class("preview")
                            .class(|| (CLASS_HTML_PREVIEW, true))
                            .class(move || (CLASS_HIDDEN, !preview.get()))
                            .inner_html(move || preview.get().then(|| md_html(&codemirror_read()))),
                        div()
                            .id("editor")
                            .class("editor")
                            .class(move || (CLASS_HIDDEN, preview.get())),
                    )),
                ),
                div()
                    .class(CLASS_ML_ACTIONS)
                    .class(|| ("side", true))
                    .child((
                        div().child((
                            button()
                                .disabled(move || upload_progress.get() > 0)
                                .title("добавить файл")
                                .child(move || {
                                    let v = upload_progress.get();
                                    if v > 0 {
                                        v.to_string().into_any()
                                    } else {
                                        svg_icon(ADD_FILE_SVG).into_any()
                                    }
                                })
                                .class("attachment-upload__button")
                                .child(move || {
                                    (upload_progress.get() == 0).then_some(move || {
                                        input()
                                            .class("attachment-upload__input")
                                            .r#type("file")
                                            .multiple(true)
                                            .on(ev::change, move |e| {
                                                action_upload(e, upload_progress)
                                            })
                                    })
                                }),
                            button()
                                .title("предпросмотр")
                                .child(svg_icon(PREVIEW_SVG))
                                .on(ev::click, move |_| {
                                    set_preview.set(!preview.get());
                                }),
                        )),
                        div().child((
                            button().child("закрыть").on(ev::click, |_| {
                                editor_close();
                            }),
                            button()
                                .class(CLASS_CONFIRM)
                                .child("отправить")
                                .disabled(move || send_disabled.get())
                                .on(ev::click, move |_| {
                                    action_send(recipient, subject, send_disabled, attachments)
                                }),
                        )),
                    )),
            )),
        )
        .into_any()
}

fn editor_close() {
    APP_EDITOR.set(None);
}

fn action_upload(e: Event, upload_progress: RwSignal<usize>) {
    upload_progress.set(1);
    spawn_local(async move {
        let Some(files) = html_input_from_target(e.target()).and_then(|input| input.files()) else {
            upload_progress.set(0);
            return;
        };
        upload_progress.set(2);
        let mut results: FileUpload = Vec::new();
        let all_files = files.length();
        for ind in 0..all_files {
            if let Some(file) = files.item(ind)
                && file.size() < 1024.0 * 1024.0 * 20.0
                && let Ok(file_data) = file_to_vec(&file).await
            {
                let file_name = file.name();
                results.push((file_name, file_data));
                upload_progress.set(upload_progress.get_untracked() + 1);
            }
        }
        if !results.is_empty() {
            upload_progress.set(upload_progress.get_untracked() + 1);
            send_data_conn(DataConnParams::Files(DataFileParam::Upload(results)));
        }
    });
}

fn action_send(
    recipient: RwSignal<String>,
    subject: RwSignal<String>,
    send_disabled: RwSignal<bool>,
    attachments: RwSignal<Vec<DataFileItem>>,
) {
    let field_recipient = recipient.get_untracked().trim().to_string();
    let field_subject = subject.get_untracked().trim().to_string();
    let field_content = codemirror_read().trim().to_string();
    if field_recipient.is_empty() {
        app_modal_error("Укажите получателя");
        return;
    }
    if field_subject.is_empty() {
        app_modal_error("Укажите тему");
        return;
    }
    if field_content.is_empty() {
        app_modal_error("Введите сообщение");
        return;
    }
    send_disabled.set(true);
    send_data_conn(DataConnParams::Message(DataMessageParams::Send(
        field_recipient,
        field_subject,
        field_content,
        attachments.get_untracked(),
    )));
}

const PREVIEW_SVG: &str = include_str!("../../../assets/icons/editor/eyes.svg");
const ADD_FILE_SVG: &str = include_str!("../../../assets/icons/editor/plus.svg");
