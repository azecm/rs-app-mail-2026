use crate::constants::{
    CLASS_CONFIRM, CLASS_ML_ACTIONS, CLASS_ML_BODY, CLASS_ML_CONTAINER, CLASS_ML_HEADER,
    CLASS_ML_WINDOW,
};
use crate::store::APP_MODAL;
use leptos::ev;
use leptos::html::*;
use leptos::prelude::*;

pub fn view_modal() -> impl IntoView {
    || {
        APP_MODAL.get().map(|data| {
            div().class(CLASS_ML_CONTAINER).child(
                div().class(CLASS_ML_WINDOW).child((
                    div().class(CLASS_ML_HEADER).child(data.title),
                    div().class(CLASS_ML_BODY).child(data.content),
                    div()
                        .class(CLASS_ML_ACTIONS)
                        .child(if let Some(call) = data.confirm {
                            (
                                button()
                                    .child("нет")
                                    .on(ev::click, |_| {
                                        APP_MODAL.set(None);
                                    })
                                    .into_any(),
                                button()
                                    .child("да")
                                    .class(CLASS_CONFIRM)
                                    .on(ev::click, move |_| {
                                        call();
                                        APP_MODAL.set(None);
                                    })
                                    .into_any(),
                            )
                                .into_any()
                        } else {
                            button()
                                .child("ok")
                                .on(ev::click, |_| {
                                    APP_MODAL.set(None);
                                })
                                .into_any()
                        }),
                )),
            )
        })
    }
}
