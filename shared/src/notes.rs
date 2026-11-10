use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataNoteResults {
    pub idn: i32,
    pub idp: Option<i32>,
    pub position: Option<i32>,
    pub label: Option<String>,
    pub email: Option<String>,
    pub content: Option<String>,
    pub event: Option<Option<DataNoteEvent>>,
    pub to: Option<i32>,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Serialize, Default, Deserialize)]
pub struct DataNoteEvent {
    pub date: String,
    pub delta: i32,
    pub period: i32,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default)]
pub struct DataNoteParams {
    pub idn: i32,
    pub idp: Option<i32>,
    pub position: Option<i32>,
    pub label: Option<String>,
    pub email: Option<String>,
    pub content: Option<String>,
    pub event: Option<DataNoteEvent>,
    // --
    pub from: Option<i32>,
    pub to: Option<i32>,
    pub action: DataNoteAction,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default, Copy, Deserialize, Serialize)]
pub enum DataNoteAction {
    Insert,
    #[default]
    Update,
    Remove,
    Position(i32),
}
