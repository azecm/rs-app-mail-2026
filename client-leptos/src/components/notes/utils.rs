use crate::constants::CLASS_HIDDEN;
use crate::store::APP_IS_MOBILE;
use crate::store::APP_NOTES_EDITING;
use crate::store::APP_NOTES_ROWS;
use crate::store::EventItemStruct;
use leptos::prelude::*;
use shared::DataNoteResults;
use std::collections::{HashMap, HashSet};
use web_sys::js_sys;

pub type CurrentIdn = RwSignal<i32>;
pub type NotesMap = HashMap<i32, Vec<i32>>;
pub type ReadNotesMap = ReadSignal<NotesMap>;

pub fn update_counts() -> usize {
    APP_NOTES_ROWS
        .get()
        .values()
        .filter(|row| row.idp != Some(0))
        .count()
}

pub fn use_notes_map() -> ReadNotesMap {
    let (map, set_map) = signal::<NotesMap>(HashMap::new());
    Effect::new(move || set_map.set(update_map()));
    map
}

fn update_map() -> NotesMap {
    let mut map = APP_NOTES_ROWS.get().values().fold(
        HashMap::<i32, Vec<(i32, i32)>>::new(),
        |mut acc, row| {
            if let Some(idp) = row.idp {
                let row = (row.idn, row.position.unwrap_or_default());
                if let Some(list) = acc.get_mut(&idp) {
                    list.push(row);
                } else {
                    acc.insert(idp, vec![row]);
                }
            }
            acc
        },
    );

    map.iter_mut()
        .map(|(a, b)| {
            b.sort_by_key(|a| a.1);
            (*a, b.iter().map(|v| v.0).collect::<Vec<i32>>())
        })
        .collect::<NotesMap>()
}

pub fn update_events() -> Vec<EventItemStruct> {
    let delta = 1000.0 * 3600.0 * 24.0 * 3.0;
    let groups = APP_NOTES_ROWS
        .get()
        .values()
        .filter(|row| row.idp == Some(0))
        .map(|item| (item.idn, item.label.clone().unwrap_or_default()))
        .collect::<HashMap<_, _>>();

    let mut items = APP_NOTES_ROWS
        .get()
        .values()
        .filter_map(|note| match note.event.clone() {
            Some(Some(event)) => {
                let js_date: js_sys::Date = js_sys::Date::new(&event.date.clone().into());
                if js_date.get_time() - js_sys::Date::now() < delta {
                    let group = match groups.get(&note.idp.unwrap_or_default()) {
                        Some(v) => v.clone(),
                        None => "".to_string(),
                    };
                    Some(EventItemStruct {
                        idn: note.idn,
                        label: note.label.clone().unwrap_or_default(),
                        order: js_date.get_time() as i64,
                        date: event.date,
                        group,
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    items.sort_by_key(|a| a.order);
    items
}

pub fn update_email_list() -> Vec<String> {
    let mut list = Vec::from_iter(APP_NOTES_ROWS.get().values().fold(
        HashSet::<String>::new(),
        |mut set, row| {
            if let Some(email) = row.email.clone()
                && let Some(label) = row.label.clone()
                && !email.is_empty()
                && email.contains("@")
            {
                set.insert(if label.is_empty() {
                    email
                } else {
                    format!("{label} <{email}>")
                });
            }
            set
        },
    ));
    list.sort();
    list
}

pub fn row_by_idn(idn: i32) -> Option<DataNoteResults> {
    APP_NOTES_ROWS.get_untracked().get(&idn).cloned()
}

pub fn position_max(map: &NotesMap, idp: i32) -> i32 {
    if let Some(rows) = map.get(&idp) {
        rows.len() as i32
    } else {
        0
    }
}

pub fn block_hidden(flag: bool) -> (&'static str, bool) {
    let result = APP_NOTES_EDITING.get() == Some(true);
    (CLASS_HIDDEN, APP_IS_MOBILE.get() && result == flag)
}
