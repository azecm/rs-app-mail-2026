mod transport_post;
mod transport_sse;

use crate::{
    components::codemirror_close,
    constants::LOADING_FINISHED,
    dom::{download_file, get_location, local_storage_set, storage_key},
    store::{
        APP_BOXES, APP_EDITOR, APP_MESSAGES, APP_NOTE_VERSION, APP_NOTES_ROWS, APP_SESSION,
        APP_SSE_CHECKER, APP_STATE, APP_USER, APP_VERSION, AppState, AppUser,
    },
    utils::{app_modal_error, logout},
};
use leptos::reactive::traits::{GetUntracked, Set, Update};
use shared::{
    DataBoxResults, DataConnParams, DataEBox, DataFileResults, DataInitResults, DataMessageResults,
    DataParams, DataResults,
};
use std::collections::{HashMap, HashSet};
use wasm_bindgen_futures::spawn_local;

pub use self::transport_sse::{sse_init, sse_result};

pub fn send_data_sync(data: DataParams) {
    spawn_local(async move {
        send_data(data).await;
    });
}

pub fn send_data_conn(data: DataConnParams) {
    spawn_local(async move {
        send_data(DataParams::Conn(data)).await;
    });
}

async fn send_data(data: DataParams) {
    let session_id = APP_SESSION.get_untracked();
    let session_id = (!session_id.is_empty()).then_some(session_id);

    match self::transport_post::transport_post(data, session_id).await {
        DataResults::Auth(data) => match data {
            Some(data) => {
                authorized(data);
            }
            None => {
                logout();
            }
        },
        DataResults::Login(data) => match data {
            Ok(data) => {
                authorized(data);
            }
            Err(text) => {
                APP_STATE.set(AppState::Login(text));
            }
        },
        DataResults::Version(data) => version(data),
        DataResults::Init(data) => init(data),
        DataResults::NoteContent(idn, text) => note_content(idn, text),
        DataResults::Box(data) => ebox_result(data),
        DataResults::Message(data) => message_result(data),
        DataResults::Files(data) => files_result(data),
        DataResults::Error(text) => {
            if let Some(ed) = APP_EDITOR.get_untracked() {
                ed.upload_progress.set(0);
            }
            app_modal_error(&text);
        }
        DataResults::Skip => {}
    }
}

fn version(next: String) {
    //log::info!("VERSION {next} sse: {:?}", APP_SSE_CHECKER.get_untracked());
    let prev = APP_VERSION
        .lock()
        .map(|data| data.to_string())
        .unwrap_or_default();
    if prev.is_empty()
        && let Ok(mut guard) = APP_VERSION.lock()
    {
        *guard = next;
    } else if (next != prev || APP_SSE_CHECKER.get_untracked().is_some())
        && let Some(l) = get_location()
    {
        l.reload().ok();
    }
}

fn authorized((session_id, auth_id): (String, String)) {
    local_storage_set(&storage_key(), &auth_id);
    APP_SESSION.set(session_id);
    APP_STATE.set(AppState::Working);
}

fn init(data: DataInitResults) {
    APP_USER.set(AppUser {
        email: data.user.email,
        signature: data.user.signature,
    });

    if let Ok(mut map) = APP_MESSAGES.write() {
        map.extend(data.news.iter().map(|r| (r.idb, r.clone())));
    }

    if let Some(box_data) = APP_BOXES.get(&DataEBox::Incoming) {
        box_data.loading.set(LOADING_FINISHED);
        box_data.finished.set(true);
        box_data
            .list
            .update(|list| list.extend(data.news.iter().map(|r| r.idb).collect::<Vec<_>>()));
    }

    APP_NOTES_ROWS.set(HashMap::from_iter(
        data.notes.iter().map(|r| (r.idn, r.clone())),
    ));
}

fn ebox_result(data: DataBoxResults) {
    if let Some(box_data) = APP_BOXES.get(&data.ebox) {
        let finished = data.data.is_empty();
        box_data.loading.set(LOADING_FINISHED);
        box_data.finished.set(finished);

        if let Ok(mut map) = APP_MESSAGES.write() {
            map.extend(data.data.iter().map(|r| (r.idb, r.clone())));
        }

        let set: HashSet<i64> = HashSet::from_iter(box_data.list.get_untracked().iter().copied());
        let next = data.data.iter().fold(Vec::<i64>::new(), |mut acc, row| {
            if !set.contains(&row.idb) {
                acc.push(row.idb);
            }
            acc
        });

        box_data.list.update(|prev| prev.extend(next));
        /*if data.news {
            next.extend(&box_data.list.get_untracked());
            box_data.list.set(next);
        } else {
            box_data.list.update(|prev| prev.extend(next));
        }*/
    }
}

fn note_content(idn: i32, text: String) {
    let mut map = APP_NOTES_ROWS.get_untracked();
    if let Some(row) = map.get_mut(&idn) {
        row.content = Some(text);
        APP_NOTES_ROWS.set(map);
        APP_NOTE_VERSION.set(APP_NOTE_VERSION.get_untracked() + 1);
    }
}

fn message_result(data: DataMessageResults) {
    match data {
        DataMessageResults::Send(flag) => {
            if flag {
                codemirror_close();
                APP_EDITOR.set(None);
            } else if let Some(ed) = APP_EDITOR.get_untracked() {
                ed.send_disabled.set(false);
                app_modal_error("Ошибка при отправке...");
            }
        }
        DataMessageResults::Loaded(data, ebox) => {
            if let Some(box_data) = APP_BOXES.get(&ebox) {
                let mut set: HashSet<i64> =
                    HashSet::from_iter(box_data.list.get_untracked().iter().copied());
                set.insert(data.idb);

                if let Ok(mut map) = APP_MESSAGES.write() {
                    map.insert(data.idb, data);
                }

                box_data.list.set(msg_ordered(set));
            }
        }
        DataMessageResults::New(_ebox, _message) => {}
    }
}

fn msg_ordered(set: HashSet<i64>) -> Vec<i64> {
    let mut result = Vec::<i64>::new();
    if let Ok(map) = APP_MESSAGES.read() {
        let mut rows = map.values().fold(Vec::new(), |mut acc, r| {
            if set.contains(&r.idb) {
                acc.push((r.idb, r.order));
            }
            acc
        });
        rows.sort_by_key(|a| a.1);
        result = rows.iter().map(|r| r.0).collect();
    }
    result
}

fn files_result(data: DataFileResults) {
    match data {
        DataFileResults::Uploaded(data) => {
            if let Some(ed) = APP_EDITOR.get_untracked() {
                ed.upload_progress.set(0);
                ed.attachments.update(|list| list.extend(data));
            }
        }
        DataFileResults::Remove(temp_file) => {
            if let Some(ed) = APP_EDITOR.get_untracked() {
                ed.attachments
                    .update(|list| list.retain(|item| item.temp_file() != temp_file));
            }
        }
        DataFileResults::Download(data) => {
            download_file(data);
        }
    }
}
