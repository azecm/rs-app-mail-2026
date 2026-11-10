use crate::db::t_boxes::db_box_news;
use crate::db::{t_notes::db_notes_select, t_user::db_user_by_idu};
use crate::points::box_data::box_messages;
use shared::DataInitResults;
use shared::DataUser;
use shared::{DataNoteEvent, DataNoteResults, DataResults};

pub async fn data_init(idu: i32) -> anyhow::Result<DataResults> {
    tracing::info!("data_init");
    let user = db_user_by_idu(idu).await;
    let notes = db_notes_select(idu).await;
    let news = db_box_news(idu).await;

    Ok(DataResults::Init(DataInitResults {
        user: DataUser {
            email: user.email.clone(),
            prefix: user.name.clone(),
            signature: user.signature.clone(),
        },
        notes: notes
            .iter()
            .map(|r| DataNoteResults {
                idn: r.idn,
                idp: Some(r.idp),
                content: Some(r.content.clone()),
                email: Some(r.email.clone()),
                label: Some(r.label.clone()),
                position: Some(r.position),
                event: r.event.as_ref().map(|r| {
                    Some(DataNoteEvent {
                        date: r.date.clone(),
                        delta: r.delta,
                        period: r.period,
                    })
                }),
                to: None,
            })
            .collect::<Vec<_>>(),
        news: box_messages(news),
    }))
}
