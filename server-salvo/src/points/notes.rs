use crate::db::t_notes::{
    db_notes_content, table_notes_insert, table_notes_remove, table_notes_update,
};
use shared::DataResults;
use shared::{DataNoteAction, DataNoteParams};

pub async fn data_notes(idu: i32, data: DataNoteParams) -> anyhow::Result<DataResults> {
    match data.action {
        DataNoteAction::Update => {
            table_notes_update(idu, data).await;
        }
        DataNoteAction::Insert => {
            table_notes_insert(idu, data).await;
        }
        DataNoteAction::Remove => {
            table_notes_remove(idu, data.idn).await;
        }
        DataNoteAction::Position(_) => {}
    };

    Ok(DataResults::Skip)
}

pub async fn data_note_content(idu: i32, idn: i32) -> anyhow::Result<DataResults> {
    if let Some(text) = db_notes_content(idu, idn).await {
        Ok(DataResults::NoteContent(idn, text))
    } else {
        Ok(DataResults::Error(
            "Ошибка при сохранении содержания".to_string(),
        ))
    }
}
