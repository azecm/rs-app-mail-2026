use super::init::{db_query, db_update_query};
use crate::channels::send_by_idu;
use crate::utils::ebox_box;
use bitcode::Encode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{DataEBox, SSEMessage, ServerEvent};
use std::error::Error as StdError;
use std::time::{Duration, SystemTime};
use tokio_postgres::{
    Row,
    types::{FromSql, ToSql, Type},
};

const IDB: &str = "idb";
const IDU: &str = "idu";
const BOX: &str = "box";
const DATE: &str = "date";
const SOURCE_FILE: &str = "source_file";
const SENDER: &str = "sender";
const RECIPIENT: &str = "recipient";
const SUBJECT: &str = "subject";
const CONTENT: &str = "content";
const ATTACHMENTS: &str = "attachments";
const TABLE: &str = "emails.boxes";
#[allow(unused)]
const BY_PAGE: i64 = 30;

#[derive(Debug, Clone, Encode)]
pub struct DBBox {
    pub idb: i64,
    pub date: String,
    pub order: u64,
    pub source_file: String,
    pub sender: DBMailAddress,
    pub recipient: DBMailAddress,
    pub subject: String,
    pub content: String,
    pub attachments: Option<DBMailAttachments>,
}
impl From<Row> for DBBox {
    fn from(row: Row) -> Self {
        // тк храним время без яасового пояса
        let now = SystemTime::now()
            .checked_add(Duration::from_hours(3))
            .unwrap_or_else(SystemTime::now);
        let date: SystemTime = row.try_get(DATE).unwrap_or(SystemTime::now());
        let delta = now
            .duration_since(date)
            .map(|r| r.as_secs())
            .unwrap_or_default();

        let datetime: DateTime<Utc> = date.into();
        let datetime = datetime.format("%d.%m.%Y %T").to_string();

        Self {
            idb: row.try_get(IDB).unwrap_or_default(),
            date: datetime,
            //order: date.elapsed().unwrap_or_default().as_secs(),
            order: delta,
            source_file: row.try_get(SOURCE_FILE).unwrap_or_default(),
            sender: row.try_get(SENDER).unwrap_or_default(),
            recipient: row.try_get(RECIPIENT).unwrap_or_default(),
            subject: row.try_get(SUBJECT).unwrap_or_default(),
            content: row.try_get(CONTENT).unwrap_or_default(),
            attachments: row.try_get(ATTACHMENTS).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Default, Encode, Deserialize, Serialize)]
pub struct DBMailAddress {
    pub name: Option<String>,
    pub address: String,
}
impl<'a> FromSql<'a> for DBMailAddress {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<DBMailAddress, Box<dyn StdError + Send + Sync + 'static>> {
        match serde_json::from_slice::<DBMailAddress>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBMailAddress {:?}", err);
                Ok(DBMailAddress::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}

#[derive(Debug, Clone, Encode, Deserialize, Default, Serialize)]
pub struct DBMailAttachments {
    pub key: String,
    pub list: Vec<DBMailAttachmentItem>,
}
impl<'a> FromSql<'a> for DBMailAttachments {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<DBMailAttachments, Box<dyn StdError + Send + Sync + 'static>> {
        match serde_json::from_slice::<DBMailAttachments>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBMailAttachments {:?}", err);
                Ok(DBMailAttachments::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}

#[derive(Debug, Clone, Encode, Default, Deserialize, Serialize)]
pub struct DBMailAttachmentItem {
    pub id: usize,
    #[serde(alias = "fileName")]
    pub file_name: String,
    pub size: u64,
}
impl<'a> FromSql<'a> for DBMailAttachmentItem {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<DBMailAttachmentItem, Box<dyn StdError + Send + Sync + 'static>> {
        match serde_json::from_slice::<DBMailAttachmentItem>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBMailAttachmentItem {:?}", err);
                Ok(DBMailAttachmentItem::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}

#[allow(unused)]
pub async fn db_box_last(idu: i32) -> Vec<DBBox> {
    let email_box: i32 = 0;
    let q = format!(
        "select {IDB},{DATE},{SENDER},{RECIPIENT},{SUBJECT},{CONTENT},{ATTACHMENTS},{SOURCE_FILE} from {TABLE} where {IDU}={idu} and {BOX}={email_box} order by {DATE} desc;"
    );
    db_query(DBBox::from, &q, &[]).await
}

pub async fn db_box_page(idu: i32, ebox: DataEBox, page: usize) -> Vec<DBBox> {
    let email_box = ebox_box(ebox);
    let offset: i64 = (page as i64) * BY_PAGE;
    let q = format!(
        "select {IDB},{DATE},{SENDER},{RECIPIENT},{SUBJECT},{CONTENT},{ATTACHMENTS},{SOURCE_FILE} from {TABLE} where {IDU}={idu} and {BOX}={email_box} order by {DATE} desc offset {offset} limit {BY_PAGE};"
    );
    db_query(DBBox::from, &q, &[]).await
}

pub async fn db_box_news(idu: i32) -> Vec<DBBox> {
    let email_box = 0;
    let q = format!(
        "select {IDB},{DATE},{SENDER},{RECIPIENT},{SUBJECT},{CONTENT},{ATTACHMENTS},{SOURCE_FILE} from {TABLE} where {IDU}={idu} and {BOX}={email_box} order by {DATE} desc;"
    );
    db_query(DBBox::from, &q, &[]).await
}

pub async fn db_message(idu: i32, idb: i64) -> Option<DBBox> {
    let q = format!(
        "select {IDB},{DATE},{SENDER},{RECIPIENT},{SUBJECT},{CONTENT},{ATTACHMENTS},{SOURCE_FILE} from {TABLE} where {IDU}={idu} and {IDB}={idb};"
    );
    db_query(DBBox::from, &q, &[]).await.first().cloned()
}

pub async fn db_message_move(idu: i32, idb: i64, ebox: DataEBox) -> bool {
    let box_target = ebox_box(ebox);
    db_update_query(
        &format!("update {TABLE} set {BOX}={box_target} where {IDB}={idb} and {IDU}={idu};"),
        &[],
    )
    .await
}

pub async fn db_message_source(idu: i32, idb: i64) -> Option<String> {
    let q = format!("select {SOURCE_FILE} from {TABLE} where {IDU}={idu} and {IDB}={idb};");
    db_query(DBBox::from, &q, &[])
        .await
        .first()
        .and_then(|data| (!data.source_file.is_empty()).then_some(data.clone().source_file.clone()))
}

#[allow(clippy::too_many_arguments)]
pub async fn db_message_add(
    idu: i32,
    ebox: DataEBox,
    source_file: String,
    sender: DBMailAddress,
    recipient: DBMailAddress,
    subject: String,
    content: String,
    attachments: Option<DBMailAttachments>,
) {
    let mut fields: Vec<String> = Vec::new();
    let mut linked: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    if let Ok(txt) = serde_json::to_string(&sender) {
        fields.push(SENDER.to_string());
        values.push(format!("$${}$$", txt));
    }

    if let Ok(txt) = serde_json::to_string(&recipient) {
        fields.push(RECIPIENT.to_string());
        values.push(format!("$${}$$", txt));
    }

    if let Some(attachments) = attachments
        && let Ok(txt) = serde_json::to_string(&attachments)
    {
        fields.push(ATTACHMENTS.to_string());
        values.push(format!("$${}$$", txt));
    }

    fields.push(SUBJECT.to_string());
    linked.push(subject);
    values.push(format!("${}", linked.len()));

    fields.push(CONTENT.to_string());
    linked.push(content);
    values.push(format!("${}", linked.len()));

    fields.push(SOURCE_FILE.to_string());
    linked.push(source_file);
    values.push(format!("${}", linked.len()));

    fields.push(IDU.to_string());
    values.push(idu.to_string());

    fields.push(BOX.to_string());
    values.push(ebox_box(ebox).to_string());

    let prepared_linked = linked
        .iter()
        .map(|a| a as &(dyn ToSql + Sync))
        .collect::<Vec<_>>();

    let rows = db_query(DBBox::from, &format!("insert into {TABLE} ({}) values ({}) returning {IDB}, {DATE}, {SOURCE_FILE}, {SENDER}, {RECIPIENT}, {SUBJECT}, {CONTENT}, {ATTACHMENTS};", fields.join(","), values.join(",")), &prepared_linked[..]).await;

    if let Some(idb) = rows.last().map(|data| data.idb) {
        send_by_idu(idu, ServerEvent::Message(SSEMessage::Load(idb, ebox))).await;
    }
}

/*
pub async fn db_messages_move() {
    let email_box = ebox_box(DataEBox::Saved);
    let rows = db_query(
        DBBox::from,
        &format!("select {IDB} from {TABLE} where {BOX}={email_box};"),
        &[],
    )
    .await;
    let box_target = ebox_box(DataEBox::Deleted);
    for row in rows {
        let idb = row.idb;
        db_update_query(
            &format!("update {TABLE} set {BOX}={box_target} where {IDB}={idb};"),
            &[],
        )
        .await;
    }
}


pub async fn db_messages_convert() {
    let rows = db_query(DBBox::from, &format!("select {IDB} from {TABLE};"), &[]).await;
    for row_idb in rows {
        if let Some(row) = db_query(
            DBBox::from,
            &format!("select {CONTENT} from {TABLE} where {IDB}={};", row_idb.idb),
            &[],
        )
        .await
        .first()
        {
            db_content_update(row_idb.idb, &row.content).await;
        }
    }
}

async fn db_content_update(idb: i64, content: &str) {
    let mut fields: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    fields.push(format!("{CONTENT}=${}", fields.len() + 1));
    values.push(content.to_string());

    let prepared_values = values
        .iter()
        .map(|a| a as &(dyn ToSql + Sync))
        .collect::<Vec<_>>();

    let _ = db_update_query(
        &format!("update {TABLE} set {} where {IDB}={idb};", fields.join(",")),
        &prepared_values[..],
    )
    .await;

    //println!("RESULT: {flag}");
}
*/
