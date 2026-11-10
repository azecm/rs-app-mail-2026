use crate::components::{view_content, view_editor, view_header, view_review, view_search};
use crate::constants::{LOADING_NEED, LOADING_STARTED};
use crate::store::{APP_BOXES, APP_IS_MOBILE, APP_SSE_CHECKER, APP_VIEW, AppView};
use crate::transport::{send_data_conn, sse_init, sse_result};
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_use::{use_document_visibility, use_media_query};
use shared::{DataBoxParams, DataConnParams, DataEBox, ServerEvent};
use web_sys::VisibilityState;

pub fn app_working() -> AnyView {
    let is_mobile = use_media_query("(max-width: 767px)");
    Effect::new(move || {
        APP_IS_MOBILE.set(is_mobile.get());
    });
    //let is_dark_preferred = use_media_query("(prefers-color-scheme: dark)");

    let visibility = use_document_visibility();
    Effect::new(move || {
        if visibility.get() == VisibilityState::Visible {
            let sse_key = uuid::Uuid::new_v4().to_string();
            APP_SSE_CHECKER.set(Some(sse_key.clone()));
            send_data_conn(DataConnParams::Version(sse_key));
        }
    });

    send_data_conn(DataConnParams::Init);

    Effect::new(|| {
        let app_view = APP_VIEW.get();

        let ebox = match app_view {
            AppView::Incoming => Some(DataEBox::Incoming),
            AppView::Saved => Some(DataEBox::Saved),
            AppView::Sent => Some(DataEBox::Sent),
            AppView::Deleted => Some(DataEBox::Deleted),
            AppView::Notes => None,
        };

        if let Some((ebox_data, ebox)) = ebox.and_then(|ebox| APP_BOXES.get(&ebox).zip(Some(ebox)))
            && let page = ebox_data.page.get()
            && ebox_data.loading.get_untracked() == LOADING_NEED
            && !ebox_data.finished.get_untracked()
        {
            ebox_data.loading.set(LOADING_STARTED);
            send_data_conn(DataConnParams::Box(DataBoxParams { page, ebox }));
        }
    });

    div()
        .class("block-main")
        .child((
            div().class("block-header").child(view_header()),
            div().class("block-content").child(view_content()),
            view_review(),
            view_editor(),
            view_search(),
            app_sse(),
        ))
        .into_any()
}

fn app_sse() -> impl IntoView {
    let (sse_data, set_data) = signal::<Option<ServerEvent>>(None);
    Effect::new(move || {
        spawn_local(async move {
            sse_init(set_data).await;
        });
    });

    Effect::new(move || {
        if let Some(data) = sse_data.get() {
            sse_result(data);
        }
    });
}
