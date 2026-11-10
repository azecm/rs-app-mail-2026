use crate::points::box_data::data_box;
use crate::points::files::data_files;
use crate::points::init::data_init;
use crate::points::login::{data_auth, data_join, data_login};
use crate::points::messages::data_messages;
use crate::points::notes::{data_note_content, data_notes};
use crate::points::version::data_version;
use crate::session::get_session;
use crate::utils::data_to_bytes;
use bytes::Bytes;
use shared::decrypt_data;
use shared::{DataConnParams, DataParams, DataResults, TEST_KEY};

pub async fn data_router(bytes: &[u8], browser: String) -> anyhow::Result<Bytes> {
    let (data, params) = decrypt_data::<DataParams>(bytes, &TEST_KEY)?;

    let result = match data {
        DataParams::Login(data) => data_login(data, browser, &params).await,
        DataParams::Auth(session, email) => data_auth((session, email), browser, &params).await,
        DataParams::Conn(data) => {
            let session = if let Some(session) = params.session.clone() {
                get_session(&session, &browser, &params.hardware).await
            } else {
                None
            };
            match session {
                Some(session) => match data {
                    DataConnParams::Version(sse_key) => data_version(session.idu, sse_key).await,
                    DataConnParams::Init => data_init(session.idu).await,
                    DataConnParams::Notes(data) => data_notes(session.idu, data).await,
                    DataConnParams::NoteContent(id) => data_note_content(session.idu, id).await,
                    DataConnParams::Join(session_id, session_sse) => {
                        data_join(session_id, session_sse).await
                    }
                    DataConnParams::Box(data) => data_box(session.idu, data).await,
                    DataConnParams::Message(data) => data_messages(session.idu, data).await,
                    DataConnParams::Files(data) => data_files(session.idu, data).await,
                },
                None => Ok(DataResults::Skip),
            }
        }
    }?;

    Ok(data_to_bytes(&result))
}

/*
pub fn data_ping(state: TransportState) -> anyhow::Result<Bytes> {
    let awake = state.awake.load(Ordering::Relaxed);
    state.awake.store(awake + 1, Ordering::Relaxed);
    let ping_bytes = encrypt_to_bytes(DataResults::ServerEvent(ServerEvent::Ping))
        .map_err(|e| anyhow::anyhow!("ping_bytes {e:?}"))?;
    Ok(ping_bytes)
}

fn data_pong(state: TransportState) -> anyhow::Result<DataResults> {
    let awake = state.awake.load(Ordering::Relaxed);
    state.awake.store(awake - 1, Ordering::Relaxed);
    Ok(DataResults::Skip)
}
*/
