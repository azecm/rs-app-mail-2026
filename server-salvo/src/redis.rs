use crate::types::Session;
use redis::aio::ConnectionManager;
use redis::{AsyncTypedCommands, Client, RedisResult};
use tokio::sync::OnceCell;

const REDIS_PARAM: &str = "redis://127.0.0.1/";
const HOST: &str = "iced-mail";
const SESSION: &str = "session";
//const DOWNLOAD: &str = "download";
const DAY_PERIOD: i64 = 24 * 3600;

static CLIENT: OnceCell<Client> = OnceCell::const_new();

async fn get_pool() -> Client {
    Client::open(REDIS_PARAM).expect("Can't create Redis client")
}

async fn get_conn() -> RedisResult<ConnectionManager> {
    CLIENT
        .get_or_init(get_pool)
        .await
        .get_connection_manager()
        .await
}

fn get_key(part: &str, key: &str) -> String {
    format!("{HOST}:{part}:{key}")
}

pub fn get_redis_last(val: &str) -> String {
    val.split(':').next_back().unwrap().to_string()
}

pub async fn redis_keys(pattern: &str) -> Option<Vec<String>> {
    match get_conn().await {
        Ok(mut conn) => conn.keys(pattern).await.ok(),
        Err(err) => {
            tracing::debug!("redis_del {}", err);
            None
        }
    }
}

async fn redis_get(prefix: &str, k: &str) -> Option<String> {
    match get_conn().await {
        Ok(mut conn) => match conn.get(get_key(prefix, k)).await {
            Ok(Some(data)) => Some(data),
            _ => None,
        },
        Err(err) => {
            tracing::debug!("redis_get {}", err);
            None
        }
    }
}

async fn redis_del(prefix: &str, k: &str) -> Option<usize> {
    match get_conn().await {
        Ok(mut conn) => conn.del(get_key(prefix, k)).await.ok(),
        Err(err) => {
            tracing::debug!("redis_del {}", err);
            None
        }
    }
}

async fn redis_expire_set(prefix: &str, k: &str, value: &str, sec: i64) -> bool {
    match Client::open(REDIS_PARAM).and_then(|client| client.get_connection()) {
        Ok(mut con) => {
            let k = get_key(prefix, k);
            redis::pipe()
                .set(&k, value)
                .expire(&k, sec)
                .exec(&mut con)
                .is_ok()
        }
        Err(err) => {
            tracing::debug!("redis_expire_set {}", err);
            false
        }
    }
}

// --

/*
pub async fn redis_dl_file_set(k: &str, val: &DownloadStruct) -> bool {
    if let Ok(text) = toml::to_string(val) {
        redis_expire_set(DOWNLOAD, k, &text, 5).await
    } else {
        false
    }
}

pub async fn redis_dl_file_get(k: &str) -> Option<DownloadStruct> {
    redis_get(DOWNLOAD, k)
        .await
        .and_then(|text| toml::from_str::<DownloadStruct>(&text).ok())
}
*/

// --

pub async fn redis_session_get(k: &str) -> Option<Session> {
    redis_get(SESSION, k)
        .await
        .and_then(|text| toml::from_str::<Session>(&text).ok())
}

pub async fn redis_session_set(k: &str, session: &Session, days: i64) -> bool {
    if let Ok(text) = toml::to_string(session) {
        redis_expire_set(SESSION, k, &text, DAY_PERIOD * days).await
    } else {
        false
    }
}

pub async fn redis_session_drop(k: &str) {
    redis_del(SESSION, k).await;
}

pub async fn redis_sessions_all() -> Vec<String> {
    redis_keys(&get_key(SESSION, "*")).await.unwrap_or_default()
}

// --
