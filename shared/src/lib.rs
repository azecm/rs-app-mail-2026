use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

mod auth;
mod crypto;
mod files;
mod init;
mod login;
mod messages;
mod notes;
mod user;

pub use auth::*;
pub use crypto::*;
pub use files::*;
pub use init::*;
pub use login::*;
pub use messages::*;
pub use notes::*;
pub use user::*;

pub const VERSION: u32 = 1;
pub const TEST_KEY: [u8; 32] = [1; 32];
pub const URL_TRANSPORT: &str = "api/transport";
pub const URL_SSE: &str = "api/sse";
pub const SSE_CHANNEL: &str = "data";
pub const URL_WEB_TRANSPORT: &str = "web-transport";

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct Transport(pub Vec<u8>, pub Vec<u8>);

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct TransportContainer {
    pub data: Vec<u8>,
    pub param: TransportParams,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct TransportParams {
    pub hardware: [u8; 32],
    pub session: Option<String>,
    pub version: u32,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataParams {
    Auth(String, String),
    Login(DataLoginParams),
    Conn(DataConnParams),
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataConnParams {
    Init,
    Version(String),
    Notes(DataNoteParams),
    NoteContent(i32),
    Join(String, String),
    Box(DataBoxParams),
    Message(DataMessageParams),
    Files(DataFileParam),
    // Last,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataResults {
    Version(String),
    Login(Result<(String, String), String>),
    Auth(Option<(String, String)>),
    Init(DataInitResults),
    NoteContent(i32, String),
    Box(DataBoxResults),
    Message(DataMessageResults),
    Files(DataFileResults),
    Error(String),
    Skip,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum ServerEvent {
    Initialized(String),
    NoteUpdated(SSENote),
    Message(SSEMessage),
    Checker(String),
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum SSEMessage {
    Load(i64, DataEBox),
    Moved(i64, DataEBox, DataEBox),
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum SSENote {
    Content(i32),
    Inserted(DataNoteResults),
    Updated(DataNoteResults),
    Position(i32, i32),
    Remove(i32),
}
