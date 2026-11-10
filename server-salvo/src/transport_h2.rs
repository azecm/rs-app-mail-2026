use crate::channels::set_channel;
use crate::points::_router::data_router;
use crate::utils::data_to_bytes;
use futures_util::StreamExt;
use salvo::http::ResBody;
use salvo::sse::{SseEvent, SseKeepAlive};
use salvo::{Request, Response, handler};
use shared::{DataResults, SSE_CHANNEL, ServerEvent};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[handler]
pub async fn transport_sse(res: &mut Response) {
    let id = uuid::Uuid::new_v4().to_string();

    tracing::info!("new chat user: {}", id);

    let (session_sender, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    session_sender
        .send(ServerEvent::Initialized(id.clone()))
        .ok();

    set_channel(&id, session_sender).await;

    let stream = rx.map(|msg| {
        let text_data = toml::to_string(&msg).unwrap_or_default();
        Ok::<_, salvo::Error>(SseEvent::default().name(SSE_CHANNEL).text(text_data))
    });

    /*
    salvo::extra::sse::stream(res, event_stream).keep_alive(
        // The client will automatically try to reconnect if the connection drops
        sse::KeepAlive::default().interval(std::time::Duration::from_secs(1))
    );
    */

    // https://salvo.rs/guide/features/sse

    SseKeepAlive::new(stream).stream(res);
}

/*
#[handler]
pub async fn download_file(req: &mut Request) -> salvo::Result<NamedFile> {
    let file_id = req.param::<String>("id").unwrap_or_default();
    let file = redis_dl_file_get(&file_id).await.unwrap_or_default();

    NamedFile::builder(file.path)
        .attached_name(file.name)
        .use_etag(true)
        .build()
        .await
}
*/

#[handler]
pub async fn transport_data(req: &mut Request, res: &mut Response) {
    let browser = req
        .headers()
        .get("user-agent")
        .map(|v| v.to_str().unwrap_or("unknown"))
        .unwrap_or("unknown")
        .to_string();

    //tracing::info!("transport_data");

    req.set_secure_max_size(1024 * 1024 * 10);

    let bytes = match req.payload().await {
        Ok(bytes) => data_router(bytes, browser).await.unwrap_or_else(|e| {
            tracing::error!("transport_data::data_router {}", e);
            bytes::Bytes::new()
        }),
        Err(e) => {
            tracing::error!("transport_data::payload {}", e);
            data_to_bytes(&DataResults::Error(format!("Ошибка: {e}")))
        }
    };

    res.body(ResBody::Once(bytes));
}
