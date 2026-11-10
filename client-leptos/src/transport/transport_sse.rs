use crate::store::{APP_BOXES, APP_NOTES_ROWS, APP_SESSION, APP_SSE_CHECKER};
use crate::transport::{msg_ordered, send_data_conn};
use crate::utils::app_modal_error;
use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use leptos::reactive::signal::WriteSignal;
use leptos::reactive::traits::{GetUntracked, Set, Update};
use shared::{
    DataConnParams, DataMessageParams, DataNoteResults, SSE_CHANNEL, SSEMessage, SSENote,
    ServerEvent, URL_SSE,
};
use std::collections::HashSet;
use std::time::Duration;
use wasmtimer::tokio::sleep;

pub async fn sse_init(set_data: WriteSignal<Option<ServerEvent>>) {
    loop {
        log::info!("EventSource::new");
        match EventSource::new(URL_SSE) {
            Ok(mut source) => match source.subscribe(SSE_CHANNEL) {
                Ok(mut stream) => {
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok((_, msg)) => {
                                if let Some(msg) = msg
                                    .data()
                                    .as_string()
                                    .and_then(|str| toml::from_str::<ServerEvent>(&str).ok())
                                {
                                    set_data.set(Some(msg));
                                }
                            }
                            Err(e) => {
                                log::error!("sse disconnected [0] {e:?}");
                                break;
                            }
                        }
                        log::info!("sse::NEXT");
                    }
                    log::info!("sse [1]");
                }
                Err(e) => {
                    app_modal_error("sse connecting error [1]");
                    log::error!("sse connecting error [1] {e:?}");
                    sleep(Duration::from_secs(1)).await;
                }
            },
            Err(e) => {
                app_modal_error("sse connecting error [2]");
                log::error!("sse connecting error [2] {e:?}");
                sleep(Duration::from_secs(1)).await;
            }
        }
        log::info!("sse [2]");
    }
}

pub fn sse_result(result: ServerEvent) {
    //log::info!("sse result: {:?}", result);
    match result {
        ServerEvent::Initialized(sse_id) => {
            send_data_conn(DataConnParams::Join(APP_SESSION.get_untracked(), sse_id));
        }
        ServerEvent::NoteUpdated(result) => {
            sse_notes(result);
        }
        ServerEvent::Message(result) => {
            sse_message(result);
        }
        ServerEvent::Checker(sse_key) => {
            if APP_SSE_CHECKER.get_untracked() == Some(sse_key) {
                APP_SSE_CHECKER.set(None);
            }
        }
    }
}

fn sse_message(data: SSEMessage) {
    match data {
        SSEMessage::Load(idb, ebox) => {
            send_data_conn(DataConnParams::Message(DataMessageParams::Load(idb, ebox)));
        }
        SSEMessage::Moved(idb, ebox_from, ebox_to) => {
            if let Some(box_data) = APP_BOXES.get(&ebox_from) {
                box_data
                    .list
                    .update(|list| list.retain(|row_idb| row_idb != &idb));
            }
            if let Some(box_data) = APP_BOXES.get(&ebox_to) {
                let mut set: HashSet<i64> =
                    HashSet::from_iter(box_data.list.get_untracked().iter().copied());
                set.insert(idb);
                box_data.list.set(msg_ordered(set));
            }
        }
    }
}

pub fn sse_notes(result: SSENote) {
    match result {
        SSENote::Content(idn) => {
            send_data_conn(DataConnParams::NoteContent(idn));
        }
        SSENote::Updated(next_row) => {
            let mut notes = APP_NOTES_ROWS.get_untracked();
            if let Some(row) = notes.get_mut(&next_row.idn) {
                if next_row.label.is_some() {
                    row.label = next_row.label.clone();
                }
                if next_row.email.is_some() {
                    row.email = next_row.email.clone();
                }
                if next_row.position.is_some() {
                    row.position = next_row.position;
                }
                if next_row.event.is_some() {
                    if next_row
                        .event
                        .clone()
                        .flatten()
                        .unwrap_or_default()
                        .date
                        .is_empty()
                    {
                        row.event = Some(None);
                    } else {
                        row.event = next_row.event.clone();
                    }
                }
                if next_row.idp.is_some() {
                    row.idp = next_row.idp;
                }
                if next_row.to.is_some() {
                    row.idp = next_row.to;
                }
                APP_NOTES_ROWS.set(notes);
            }
        }
        SSENote::Inserted(next_row) => {
            let mut notes = APP_NOTES_ROWS.get_untracked();
            notes.insert(
                next_row.idn,
                DataNoteResults {
                    idp: if next_row.idp.is_some() {
                        next_row.idp
                    } else {
                        Some(0)
                    },
                    ..next_row
                },
            );
            APP_NOTES_ROWS.set(notes);
        }
        SSENote::Position(idn, position) => {
            let mut notes = APP_NOTES_ROWS.get_untracked();
            if let Some(row) = notes.get_mut(&idn) {
                row.position = Some(position);
                APP_NOTES_ROWS.set(notes);
            }
        }
        SSENote::Remove(id) => {
            let mut notes = APP_NOTES_ROWS.get_untracked();
            notes.remove(&id);
            APP_NOTES_ROWS.set(notes);
        }
    }
}
