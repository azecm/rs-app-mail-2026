use crate::redis::{redis_session_drop, redis_session_get, redis_session_set};
use crate::types::Session;
use std::cmp;

pub async fn init_session(idu: i32, browser: String, hardware: &[u8; 32]) -> (String, String) {
    let auth_id = uuid::Uuid::new_v4().to_string();
    let session_id = uuid::Uuid::new_v4().to_string();

    let session = Session::new(idu, browser, *hardware);

    redis_session_set(&session_id, &session.active(), 1).await;
    redis_session_set(&auth_id, &session.auth(), 30).await;

    (session_id, auth_id)
}

pub async fn drop_session(id: &str) {
    redis_session_drop(id).await;
}

/*
pub async fn init_session(idu: i32, browser: String, hardware: &[u8; 32]) -> String {
    let id = uuid::Uuid::new_v4().to_string();

    let session = Session {
        idu,
        browser,
        device: hardware.clone(),
    };

    redis_session_set(&id, &session).await;

    id
}
*/

pub async fn get_session(session: &str, browser: &str, hardware: &[u8; 32]) -> Option<Session> {
    if let Some(session) = redis_session_get(session).await {
        if session.device == *hardware && diff_browser(&session.browser, browser) < 5 {
            return Some(session);
        } else {
            tracing::error!("session_status::browser");
        }
    } else {
        tracing::error!("session_status::redis {session}");
    }
    None
}

fn diff_browser(b1: &str, b2: &str) -> usize {
    let min = cmp::min(b1.len(), b2.len());
    let max = cmp::max(b1.len(), b2.len());
    let mut result = max - min;

    for i in 0..min {
        if b1.chars().nth(i) != b2.chars().nth(i) {
            result += 1;
        }
    }

    result
}
