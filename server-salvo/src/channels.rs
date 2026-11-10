use crate::redis::{get_redis_last, redis_session_get, redis_sessions_all};
use crate::session::drop_session;
use shared::ServerEvent;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::{OnceCell, mpsc};
pub type Channels = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<ServerEvent>>>>;
static CHANNEL_MAP: OnceCell<Channels> = OnceCell::const_new();

async fn channels_map<'a>() -> &'a Channels {
    CHANNEL_MAP
        .get_or_init(|| async { Channels::default() })
        .await
}

pub async fn set_channel(session: impl Into<String>, sender: mpsc::UnboundedSender<ServerEvent>) {
    match channels_map().await.try_write() {
        Ok(mut map) => {
            map.insert(session.into(), sender);
        }
        Err(err) => {
            println!("set_channel::err {:?}", err);
        }
    }
}

pub async fn update_channel(session_id: String, session_sse: String) {
    if let Ok(mut map) = channels_map().await.try_write()
        && let Some(sender) = map.remove(&session_sse)
    {
        map.insert(session_id, sender);
    }
}

async fn drop_channels() {
    if let Ok(mut map) = channels_map().await.try_write() {
        map.retain(|_, channel| !channel.is_closed());
        println!("channels count: {}", map.iter().len());
        // sse.retain(|_uid, client| client.sender.send(Message::Reply("".to_string())).is_ok());
    }
}

pub async fn send_channel(session: impl Into<String>, data: ServerEvent) {
    match channels_map().await.try_read() {
        Ok(map) => {
            if let Some(sender) = map.get(&session.into())
                && let Err(e) = sender.send(data.clone())
            {
                tracing::error!("send_channel::send {:?} {data:?}", e);
            }
        }
        Err(err) => {
            println!("send_channel::try_read {:?}", err);
        }
    };
}

pub async fn send_by_idu(idu: i32, data: ServerEvent) {
    if idu == 0 {
        return;
    }
    for k in redis_sessions_all().await {
        let session_id = get_redis_last(&k);
        if let Some(session) = redis_session_get(&session_id).await
            && session.idu == idu
            && session.active
        {
            send_channel(session_id, data.clone()).await;
        }
    }
}

pub async fn sync_channel() {
    drop_channels().await;

    let active_sessions = channels_map()
        .await
        .try_read()
        .map(|map| map.keys().cloned().collect::<HashSet<_>>())
        .ok()
        .unwrap_or_default();

    for k in redis_sessions_all().await {
        let session_id = get_redis_last(&k);
        if let Some(session) = redis_session_get(&session_id).await
            && session.active
            && !active_sessions.contains(&session_id)
        {
            drop_session(&session_id).await;
        }
    }
}

// --
