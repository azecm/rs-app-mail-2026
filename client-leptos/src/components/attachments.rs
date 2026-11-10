use crate::transport::send_data_conn;
use leptos::{ev, html::*, prelude::*};
use shared::{DataAttachments, DataConnParams, DataFileItem, DataFileParam};

type Attachment = String;

pub fn view_attachments(data: &DataAttachments) -> impl IntoView {
    provide_context::<Attachment>(data.key.clone());
    div().class("attachments-container").child(
        data.list
            .iter()
            .map(move |row| attachment_item(row.clone()))
            .collect_view(),
    )
}

fn attachment_item(row: DataFileItem) -> impl IntoView {
    let key = use_context::<Attachment>().expect("Attachment::Ctx");
    let file_name = row.file_name.clone();
    button()
        .child((
            span()
                .class("attachment-label")
                .child(file_name.clone())
                .title(file_name.clone()),
            span().child(get_size(row.size)),
        ))
        .on(ev::click, move |_| {
            row.saved_file(&key);
            send_data_conn(DataConnParams::Files(DataFileParam::Download(
                row.saved_file(&key),
                file_name.clone(),
            )));
        })
}

pub fn editor_attachments(source: RwSignal<Vec<DataFileItem>>) -> impl IntoView {
    move || {
        let list = source.get();
        (!list.is_empty()).then_some(move || {
            div().class("attachments-container").child(
                list.iter()
                    .map(move |row| editor_item(row.clone()))
                    .collect_view(),
            )
        })
    }
}

fn editor_item(row: DataFileItem) -> impl IntoView {
    let file_name = row.file_name.clone();
    button()
        .child((
            span()
                .class("attachment-label")
                .child(file_name.clone())
                .title(file_name.clone()),
            span().child(get_size(row.size)),
        ))
        .on(ev::click, move |_| {
            send_data_conn(DataConnParams::Files(DataFileParam::Remove(
                row.temp_file(),
            )));
        })
}

pub fn get_size(size: u64) -> String {
    let mut ext = "б";
    let mut size = size as f64;
    if size > 1024.0 {
        size = (size / 1024.0 * 10.0).round() / 10.0;
        ext = "кб";
    }
    if size > 1024.0 {
        size = (size / 1024.0 * 10.0).round() / 10.0;
        ext = "мб";
    }
    if size > 1024.0 {
        size = (size / 1024.0 * 10.0).round() / 10.0;
        ext = "Гб";
    }
    format!("{size} {ext}")
}

/*
pub fn view_attachment(ui: &mut egui::Ui, data: DataAttachments, send_tx: &AppDataConn) {
    let mut clicked_link = String::new();
    let mut clicked_name = String::new();
    ui.horizontal_wrapped(|ui| {
        for item in data.list.iter() {
            ui.allocate_ui(egui::vec2(200.0, 20.0), |ui| {
                if ui
                    .add(egui::Button::new(&item.file_name).truncate())
                    .on_hover_text(&item.file_name)
                    .clicked()
                {
                    clicked_link = item.saved_file(&data.key);
                    clicked_name = item.file_name.clone();
                }
            });
            ui.label(get_size(item.size));
            ui.add_space(10.);
        }
    });
    if !clicked_link.is_empty() {
        send_tx
            .send(DataConnParams::Files(DataFileParam::Download(
                clicked_link,
                clicked_name,
            )))
            .ok();
    }
}


 */
