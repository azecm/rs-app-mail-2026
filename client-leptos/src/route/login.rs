use crate::constants::KEY_ENTER;
use crate::dom::get_email;
use crate::transport::send_data_sync;
use leptos::ev;
use leptos::html::*;
use leptos::prelude::*;
use shared::DataLoginParams;
use shared::DataParams;

pub fn app_login(msg: String) -> AnyView {
    let (user_name, set_name) = signal(String::new());
    let (user_pass, set_pass) = signal(String::new());
    div()
        .class("block-login")
        .child((
            input()
                .r#type("text")
                .class("login-input")
                .name("_")
                .on(ev::input, move |e| set_name.set(event_target_value(&e))),
            input()
                .r#type("text")
                .class("login-input")
                .name("_")
                .on(ev::input, move |e| set_pass.set(event_target_value(&e)))
                .on(ev::keydown, move |e| {
                    if e.key() == KEY_ENTER {
                        send_data_sync(DataParams::Login(DataLoginParams {
                            email: get_email(),
                            login: user_name.get().trim().to_string(),
                            pass: user_pass.get().trim().to_string(),
                        }));
                    }
                }),
            div().child(msg),
        ))
        .into_any()
}
