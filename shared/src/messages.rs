use std::fmt;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::files::DataFileItem;

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct DataBoxParams {
    pub ebox: DataEBox,
    pub page: usize,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct DataBoxResults {
    pub ebox: DataEBox,
    pub page: usize,
    pub data: Vec<DataMessageBoxed>,
}

#[derive(
    Encode, Decode, PartialEq, Debug, Copy, Clone, Eq, Hash, Deserialize, Serialize, Default,
)]
pub enum DataEBox {
    #[default]
    Incoming,
    Saved,
    Sent,
    Deleted,
}

pub type DataMessageBoxed = Box<DataMessageSrc>;

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default)]
pub struct DataMessageSrc {
    pub idb: i64,
    pub date: String,
    pub order: u64,
    pub has_source: bool,
    pub sender: DataMailAddress,
    pub recipient: DataMailAddress,
    pub subject: String,
    pub content: String,
    pub attachments: Option<DataAttachments>,
}
#[derive(Encode, Decode, PartialEq, Debug, Clone, Default)]
pub struct DataMailAddress {
    pub name: Option<String>,
    pub address: String,
}

impl fmt::Display for DataMailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(label) = self.name.clone() else {
            return write!(f, "{}", self.address);
        };
        if label.trim().is_empty() {
            write!(f, "{}", self.address)
        } else {
            write!(f, "{} <{}>", label, self.address)
        }
    }
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct DataAttachments {
    pub key: String,
    pub list: Vec<DataFileItem>,
}

// ----
#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataMessageParams {
    Move(i64, DataEBox, DataEBox),
    Load(i64, DataEBox),
    // recipient, subject, content
    Send(String, String, String, Vec<DataFileItem>),
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataMessageResults {
    Send(bool),
    Loaded(DataMessageBoxed, DataEBox),
    New(DataEBox, DataMessageBoxed),
}
