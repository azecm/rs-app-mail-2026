use super::init::{db_conn, db_query, db_update_query};
use crate::channels::send_by_idu;
use bitcode::{Decode, Encode};
use serde::Deserialize;
use shared::ServerEvent;
use shared::{DataNoteParams, DataNoteResults, SSENote};
use std::collections::HashSet;
use std::error::Error as StdError;
use tokio_postgres::{
    Row,
    types::{FromSql, ToSql, Type},
};

const IDN: &str = "idn";
const IDP: &str = "idp";
const IDU: &str = "idu";
const POSITION: &str = "position";
const LABEL: &str = "label";
const EMAIL: &str = "email";
const CONTENT: &str = "content";
const EVENT: &str = "event";
const TABLE: &str = "emails.notes";

//

#[derive(Debug, Clone, Decode, Encode)]
pub struct DBNotes {
    pub idn: i32,
    pub idp: i32,
    pub idu: i32,
    pub position: i32,
    pub label: String,
    pub email: String,
    pub content: String,
    pub event: Option<DBNotesEvent>,
}

impl From<Row> for DBNotes {
    fn from(row: Row) -> Self {
        Self {
            idn: row.try_get(IDN).unwrap_or_default(),
            idp: row.try_get(IDP).unwrap_or_default(),
            idu: row.try_get(IDU).unwrap_or_default(),
            position: row.try_get(POSITION).unwrap_or_default(),
            label: row.try_get(LABEL).unwrap_or_default(),
            email: row.try_get(EMAIL).unwrap_or_default(),
            content: row.try_get(CONTENT).unwrap_or_default(),
            event: row.try_get(EVENT).unwrap_or_default(),
        }
    }
}

// ----

#[derive(Debug, Clone, Default, Decode, Encode, Deserialize)]
pub struct DBNotesEvent {
    pub date: String,
    pub delta: i32,
    pub period: i32,
}

impl<'a> FromSql<'a> for DBNotesEvent {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<DBNotesEvent, Box<dyn StdError + Send + Sync + 'static>> {
        match serde_json::from_slice::<DBNotesEvent>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBNotesEvent {:?}", err);
                Ok(DBNotesEvent::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}

// ----

#[derive(Debug, Clone)]
pub struct DBNotesMini {
    pub idn: i32,
    pub idp: i32,
    pub position: i32,
}

impl From<Row> for DBNotesMini {
    fn from(row: Row) -> Self {
        Self {
            idn: row.get(IDN),
            idp: row.get(IDP),
            position: row.get(POSITION),
        }
    }
}

// ----

#[derive(Debug, Clone)]
pub struct DBIdn {
    pub idn: i32,
}

impl From<Row> for DBIdn {
    fn from(row: Row) -> Self {
        Self { idn: row.get(IDN) }
    }
}

// ----

pub async fn db_notes_select(idu: i32) -> Vec<DBNotes> {
    db_query(
        DBNotes::from,
        &format!("select {IDN},{IDP},{POSITION},{LABEL},{EMAIL},{CONTENT},{EVENT} from {TABLE} where {IDU}={idu} order by {IDP},{POSITION};"),
        &[],
    )
    .await
}

// ----

pub async fn table_notes_insert(idu: i32, data: DataNoteParams) {
    let mut fields: Vec<String> = Vec::new();
    let mut linked: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    if let Some(label) = &data.label {
        fields.push("label".to_string());
        linked.push(label.clone());
        values.push(format!("${}", linked.len()));
    }
    if let Some(email) = &data.email {
        fields.push("email".to_string());
        linked.push(email.clone());
        values.push(format!("${}", linked.len()));
    }
    /*
    if let Some(content) = &data.content {
        fields.push("content".to_string());
        linked.push(content.clone());
        values.push(format!("${}", linked.len()));
    }
    */

    let idp = data.idp.unwrap_or(0);
    fields.push("idp".to_string());
    values.push(idp.to_string());

    let mut prev = db_notes_position_prev(idu, idp).await;

    let position = data.position.unwrap_or((prev.len() + 1) as i32);
    fields.push("position".to_string());
    values.push(position.to_string());

    fields.push("idu".to_string());
    values.push(idu.to_string());

    if !fields.is_empty() {
        let prepared_values = linked
            .iter()
            .map(|a| a as &(dyn ToSql + Sync))
            .collect::<Vec<_>>();

        let rows = db_query(
            DBIdn::from,
            &format!(
                "insert into {TABLE} ({}) values ({}) returning {IDN};",
                fields.join(","),
                values.join(",")
            ),
            &prepared_values[..],
        )
        .await;

        if rows.len() == 1 {
            let idn = rows[0].idn;

            send_by_idu(
                idu,
                ServerEvent::NoteUpdated(SSENote::Inserted(DataNoteResults {
                    idn,
                    idp: data.idp,
                    email: data.email,
                    label: data.label,
                    position: data.position,
                    content: None,
                    event: None,
                    to: None,
                })),
            )
            .await;

            prev.insert((position - 1) as usize, DBNotesMini { idp, position, idn });
            db_notes_test_position(idu, prev).await;
        }
    }
}

pub async fn table_notes_remove(idu: i32, idn: i32) {
    let mut prev = db_notes_position_prev(idu, db_notes_idp(idu, idn).await).await;

    let rows = db_query(
        DBIdn::from,
        &format!("delete from {TABLE} where {IDU}={idu} and {IDN}={idn} returning {IDN};"),
        &[],
    )
    .await;

    if rows.len() == 1 && rows[0].idn == idn {
        send_by_idu(idu, ServerEvent::NoteUpdated(SSENote::Remove(idn))).await;

        prev.retain(|row| row.idn != idn);
        db_notes_test_position(idu, prev).await;
    }
}

pub async fn table_notes_update(idu: i32, data: DataNoteParams) {
    let idn = data.idn;
    let mut fields: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    let label = data.label;
    let email = data.email;
    let content = data.content;
    let idp = data.idp;
    let event = data.event;

    let position = if let Some(idp) = idp {
        Some((db_notes_position_prev(idu, idp).await.len() + 1) as i32)
    } else {
        data.position
    };

    // ===

    if let Some(label) = &label {
        fields.push(format!("label=${}", fields.len() + 1));
        values.push(label.clone());
    }
    if let Some(email) = &email {
        fields.push(format!("email=${}", fields.len() + 1));
        values.push(email.clone());
    }
    if let Some(content) = &content {
        fields.push(format!("content=${}", fields.len() + 1));
        values.push(content.clone());
    }
    if let Some(position) = &position {
        fields.push(format!("position={position}"));
    }
    if let Some(idp) = &idp {
        fields.push(format!("idp={idp}"));
    }
    if let Some(event_current) = event.clone() {
        if event_current.date.is_empty() {
            fields.push("event=null".to_string());
        } else if let Ok(txt) = serde_json::to_string(&event_current) {
            fields.push(format!("event=$${}$$", txt));
        }
    }
    if !fields.is_empty() {
        let mut prev = match position.is_some() || idp.is_some() {
            true => db_notes_position_prev(idu, db_notes_idp(idu, idn).await).await,
            false => vec![],
        };
        let prepared_values = values
            .iter()
            .map(|a| a as &(dyn ToSql + Sync))
            .collect::<Vec<_>>();

        if db_update_query(
            &format!(
                "update {TABLE} set {} where {IDU}={idu} and {IDN}={idn};",
                fields.join(",")
            ),
            &prepared_values[..],
        )
        .await
        {
            send_by_idu(
                idu,
                ServerEvent::NoteUpdated(SSENote::Updated(DataNoteResults {
                    idn,
                    label,
                    email,
                    content: None,
                    event: Some(event),
                    idp,
                    position,
                    to: None,
                })),
            )
            .await;
            if content.is_some() {
                send_by_idu(idu, ServerEvent::NoteUpdated(SSENote::Content(idn))).await;
            }

            if idp.is_some() {
                let current = prev.iter().position(|row| row.idn == idn);
                if let Some(current) = current {
                    prev.remove(current);
                }
                db_notes_test_position(idu, prev).await;
            } else if let Some(position) = position {
                let current = prev.iter().position(|row| row.idn == idn);
                if let Some(current) = current {
                    let item = prev.remove(current);
                    prev.insert((position - 1) as usize, item);
                }
                db_notes_test_position(idu, prev).await;
            }
        }
    }
}

pub async fn db_notes_content(idu: i32, idn: i32) -> Option<String> {
    let rows = db_query(
        DBNotes::from,
        &format!("select {CONTENT} from {TABLE} where {IDU}={idu} and {IDN}={idn};"),
        &[],
    )
    .await;

    rows.first().map(|row| row.content.clone())
}

// ----

async fn db_notes_idp(idu: i32, idn: i32) -> i32 {
    let rows = db_query(
        DBNotesMini::from,
        &format!("select {IDN}, {IDP}, {POSITION} from {TABLE} where {IDU}={idu} and {IDN}={idn};"),
        &[],
    )
    .await;
    if !rows.is_empty() { rows[0].idp } else { 0 }
}

async fn db_notes_position_prev(idu: i32, idp: i32) -> Vec<DBNotesMini> {
    db_query(
        DBNotesMini::from,
        &format!("select {IDN}, {IDP}, {POSITION} from {TABLE} where {IDU}={idu} and {IDP}={idp} order by {POSITION};"),
        &[],
    )
    .await
}

async fn db_notes_test_position(idu: i32, prev: Vec<DBNotesMini>) {
    match db_conn().await {
        Ok(db) => {
            for (ind, item) in prev.iter().enumerate() {
                let position = (ind + 1) as i32;
                if item.position != position {
                    let idn = item.idn;
                    if db
                        .query(
                            &format!("update {TABLE} set {POSITION}={position} where {IDN}={idn};"),
                            &[],
                        )
                        .await
                        .is_ok()
                    {
                        send_by_idu(
                            idu,
                            ServerEvent::NoteUpdated(SSENote::Position(idn, position)),
                        )
                        .await;
                    }
                }
            }
        }
        Err(err) => {
            tracing::error!("db_notes_test_position::err {:?}", err);
        }
    };
}

pub async fn db_trusted_domains() -> HashSet<String> {
    db_query(
        DBNotes::from,
        &format!("select {EMAIL} from {TABLE} where {EMAIL} <> '';"),
        &[],
    )
    .await
    .iter()
    .filter_map(|row| email_host(&row.email))
    .collect::<HashSet<_>>()
}

fn email_host(src: &str) -> Option<String> {
    src.split('@')
        .map(|v| v.to_lowercase())
        .collect::<Vec<_>>()
        .get(1)
        .map(|v| v.to_string())
}

/*
pub async fn db_notes_convert() {
    let rows = db_query(
        DBNotes::from,
        &format!("select {CONTENT},{IDN},{IDU} from {TABLE};"),
        &[],
    )
    .await;

    for row in rows {
        let content = row.content.trim();
        if content.is_empty() {
            continue;
        }
        table_notes_update(
            row.idu,
            DataNoteParams {
                idn: row.idn,
                content: Some(content_as_md(content).unwrap_or_default()),
                ..DataNoteParams::default()
            },
        )
        .await;
    }
}
*/
