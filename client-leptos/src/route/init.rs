use crate::dom::get_email;
use crate::dom::local_storage_get;
use crate::dom::storage_key;
use crate::store::APP_STATE;
use crate::store::AppState;
use crate::transport::send_data_sync;
use leptos::prelude::*;
use shared::DataParams;

pub fn app_init() -> AnyView {
    Effect::new(move || {
        let curent_email = get_email();
        let auth_id = local_storage_get(&storage_key()).unwrap_or_default();
        if !curent_email.is_empty() && !auth_id.is_empty() {
            send_data_sync(DataParams::Auth(auth_id.clone(), curent_email.clone()));
        } else if !curent_email.is_empty() {
            APP_STATE.set(AppState::Login(String::new()));
        } else {
            APP_STATE.set(AppState::Hello);
        }
    });

    ().into_any()
}
