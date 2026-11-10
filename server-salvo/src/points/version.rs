use base64::Engine;
use sha2::{Digest, Sha256};
use shared::DataResults;

use crate::channels::send_by_idu;

pub async fn data_version(idu: i32, sse_key: String) -> anyhow::Result<DataResults> {
    send_by_idu(idu, shared::ServerEvent::Checker(sse_key)).await;

    let data = tokio::fs::read("client-leptos/release/index.html")
        .await
        .unwrap_or_default();

    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    Ok(DataResults::Version(
        base64::engine::general_purpose::STANDARD.encode(hash),
    ))
}
