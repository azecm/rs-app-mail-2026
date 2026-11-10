use crate::channels::{sync_channel, update_channel};
use crate::db::t_user::{db_user_by_idu, db_user_login};
use crate::session::{drop_session, get_session, init_session};
use shared::{DataLoginParams, DataResults, TransportParams};

pub async fn data_login(
    data: DataLoginParams,
    browser: String,
    params: &TransportParams,
) -> anyhow::Result<DataResults> {
    let (idu, _) = db_user_login(&data.email, &data.login, &data.pass).await;
    if idu > 0 {
        Ok(DataResults::Login(Ok(init_session(
            idu,
            browser,
            &params.hardware,
        )
        .await)))
    } else {
        Ok(DataResults::Login(Err("что-то пошло не так".to_string())))
    }
}

pub async fn data_auth(
    (auth_id, email): (String, String),
    browser: String,
    params: &TransportParams,
) -> anyhow::Result<DataResults> {
    let mut result = DataResults::Auth(None);
    if let Some(session) = get_session(&auth_id, &browser, &params.hardware).await {
        let user = db_user_by_idu(session.idu).await;
        if user.email == email {
            result = DataResults::Auth(Some(
                init_session(session.idu, browser, &params.hardware).await,
            ));
        }
    }

    drop_session(&auth_id).await;

    Ok(result)
}

pub async fn data_join(session_id: String, session_sse: String) -> anyhow::Result<DataResults> {
    update_channel(session_id, session_sse).await;
    sync_channel().await;
    Ok(DataResults::Skip)
}
