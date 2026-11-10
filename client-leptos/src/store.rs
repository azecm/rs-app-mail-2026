use crate::constants::{LOADING_NEED, LOADING_STARTED};
use leptos::reactive::signal::{ArcRwSignal, RwSignal};
use shared::{DataEBox, DataFileItem, DataMessageBoxed, DataNoteResults};
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex, RwLock},
};

type MessagesMap = RwLock<HashMap<i64, DataMessageBoxed>>;
pub static APP_MESSAGES: LazyLock<MessagesMap> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub static APP_VERSION: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static APP_SSE_CHECKER: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static APP_BOXES: LazyLock<HashMap<DataEBox, BoxData>> = LazyLock::new(|| {
    HashMap::from_iter([
        (
            DataEBox::Incoming,
            BoxData {
                loading: RwSignal::new(LOADING_STARTED),
                ..BoxData::default()
            },
        ),
        (
            DataEBox::Saved,
            BoxData {
                loading: RwSignal::new(LOADING_NEED),
                ..BoxData::default()
            },
        ),
        (
            DataEBox::Sent,
            BoxData {
                loading: RwSignal::new(LOADING_NEED),
                ..BoxData::default()
            },
        ),
        (
            DataEBox::Deleted,
            BoxData {
                loading: RwSignal::new(LOADING_NEED),
                ..BoxData::default()
            },
        ),
    ])
});

type NotesRows = ArcRwSignal<HashMap<i32, DataNoteResults>>;
pub static APP_NOTES_ROWS: LazyLock<NotesRows> = LazyLock::new(|| ArcRwSignal::new(HashMap::new()));

type ModalData = ArcRwSignal<Option<AppModal>>;
pub static APP_MODAL: LazyLock<ModalData> = LazyLock::new(|| ArcRwSignal::new(None));

pub static APP_USER: LazyLock<ArcRwSignal<AppUser>> =
    LazyLock::new(|| ArcRwSignal::new(AppUser::default()));

pub static APP_STATE: LazyLock<ArcRwSignal<AppState>> =
    LazyLock::new(|| ArcRwSignal::new(AppState::default()));

pub static APP_VIEW: LazyLock<ArcRwSignal<AppView>> =
    LazyLock::new(|| ArcRwSignal::new(AppView::default()));

pub static APP_SESSION: LazyLock<ArcRwSignal<String>> =
    LazyLock::new(|| ArcRwSignal::new(String::default()));

pub static APP_IS_MOBILE: LazyLock<ArcRwSignal<bool>> = LazyLock::new(|| ArcRwSignal::new(false));

pub static APP_NOTE_VERSION: LazyLock<ArcRwSignal<usize>> = LazyLock::new(|| ArcRwSignal::new(0));
pub static APP_NOTE_FOCUS: LazyLock<ArcRwSignal<i32>> = LazyLock::new(|| ArcRwSignal::new(0));

pub static APP_NOTES_EDITING: LazyLock<ArcRwSignal<Option<bool>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static APP_SEARCH_OPENED: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(false));

pub static APP_EDITOR: LazyLock<ArcRwSignal<Option<AppEditor>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub static APP_REVIEW: LazyLock<ArcRwSignal<Option<i64>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

#[derive(Clone)]
pub struct AppModal {
    pub title: String,
    pub content: String,
    pub confirm: Option<fn()>,
}

#[derive(Default, Clone)]
pub enum AppState {
    #[default]
    Init,
    Login(String),
    Working,
    Hello,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    #[default]
    Incoming,
    Saved,
    Sent,
    Deleted,
    Notes,
}

#[derive(Default, Clone)]
pub struct AppUser {
    pub email: String,
    pub signature: String,
}

#[derive(Default, Clone)]
pub struct BoxData {
    pub list: RwSignal<Vec<i64>>,
    pub loading: RwSignal<Option<bool>>,
    pub finished: RwSignal<bool>,
    pub page: RwSignal<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct EventItemStruct {
    pub idn: i32,
    pub group: String,
    pub label: String,
    pub order: i64,
    pub date: String,
}

#[derive(Debug, Clone, Default)]
pub struct AppEditor {
    pub recipient: RwSignal<String>,
    pub subject: RwSignal<String>,
    pub content: RwSignal<String>,
    pub attachments: RwSignal<Vec<DataFileItem>>,
    pub upload_progress: RwSignal<usize>,
    pub send_disabled: RwSignal<bool>,
}
