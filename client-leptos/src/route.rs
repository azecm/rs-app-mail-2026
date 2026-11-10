mod hello;
mod init;
mod login;
mod working;

use crate::components::view_modal;
use crate::store::{APP_STATE, AppState};
use leptos::prelude::*;

pub fn app_router() -> impl IntoView {
    let result = move || match APP_STATE.get() {
        AppState::Init => self::init::app_init(),
        AppState::Login(text) => self::login::app_login(text),
        AppState::Working => self::working::app_working(),
        AppState::Hello => self::hello::app_hello(),
    };

    (result, view_modal())
}
