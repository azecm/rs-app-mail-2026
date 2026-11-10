use super::init::db_query;
use bitcode::{Decode, Encode};
use tokio_postgres::Row;

const IDU: &str = "idu";
const NAME: &str = "name";
const PASSWORD: &str = "password";
const EMAIL: &str = "email";
const SIGNATURE: &str = "signature";
const TABLE: &str = "emails.users";

#[derive(Debug, Clone, Default, Decode, Encode)]
pub struct DBUserRow {
    pub idu: i32,
    pub name: String,
    pub email: String,
    pub signature: String,
}

impl From<Row> for DBUserRow {
    fn from(row: Row) -> Self {
        Self {
            idu: row.try_get(IDU).unwrap_or_default(),
            name: row.try_get(NAME).unwrap_or_default(),
            email: row.try_get(EMAIL).unwrap_or_default(),
            signature: row.try_get(SIGNATURE).unwrap_or_default(),
        }
    }
}

pub async fn db_user_by_idu(idu: i32) -> DBUserRow {
    let rows = db_query(
        DBUserRow::from,
        &format!("select {EMAIL},{NAME},{SIGNATURE} from {TABLE} where {IDU}={idu};"),
        &[],
    )
    .await;
    if rows.len() == 1 {
        rows[0].clone()
    } else {
        DBUserRow::default()
    }
}

//

pub async fn db_all_users() -> Vec<DBUserRow> {
    db_query(
        DBUserRow::from,
        &format!("select {EMAIL},{NAME},{IDU} from {TABLE};"),
        &[],
    )
    .await
}

//

#[derive(Debug, Clone, Default)]
pub struct DBUserLogin {
    pub idu: i32,
    pub name: String,
}

impl From<Row> for DBUserLogin {
    fn from(row: Row) -> Self {
        Self {
            idu: row.get(IDU),
            name: row.get(NAME),
        }
    }
}

pub async fn db_user_login(mail_box: &str, user_name: &str, user_pass: &str) -> (i32, String) {
    let rows = db_query(
        DBUserLogin::from,
        &format!(
            "select {IDU}, {NAME} from {TABLE} where {EMAIL}=$1 and {NAME}=$2 and {PASSWORD}=$3;"
        ),
        &[&mail_box, &user_name, &user_pass],
    )
    .await;
    if rows.len() == 1 {
        (rows[0].idu, rows[0].name.to_string())
    } else {
        (0, String::new())
    }
}

/*

pub async fn db_users_convert() {
    let rows = db_query(
        DBUserRow::from,
        &format!("select {SIGNATURE},{IDU} from {TABLE};"),
        &[],
    )
    .await;

    for row in rows {
        println!("{}", row.signature);
        db_user_update(row.idu, content_as_md(&row.signature).unwrap_or_default()).await;
    }
}

async fn db_user_update(idu: i32, signature: String) {
    println!("UPDATE {idu} {signature}");
    let mut fields: Vec<String> = Vec::new();
    //let mut linked: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    fields.push(format!("{SIGNATURE}=${}", fields.len() + 1));
    values.push(signature.clone());

    let prepared_values = values
        .iter()
        .map(|a| a as &(dyn ToSql + Sync))
        .collect::<Vec<_>>();

    let flag = db_update_query(
        &format!("update {TABLE} set {} where {IDU}={idu};", fields.join(",")),
        &prepared_values[..],
    )
    .await;

    println!("RESULT: {flag}");
}
*/
