use crate::{
    db::{
        t_boxes::{db_message, db_message_source},
        t_user::db_user_by_idu,
    },
    paths::{
        attach_temp_file, file_remove, file_write, path_to_attachment, path_to_attachment_k_id,
        path_to_source,
    },
};
use shared::{DataFileItem, DataFileParam, DataFileResults, DataFileType, DataResults, FileUpload};

pub async fn data_files(idu: i32, data: DataFileParam) -> anyhow::Result<DataResults> {
    match data {
        DataFileParam::Upload(list) => Ok(file_upload(idu, list).await),
        DataFileParam::Remove(temp_file) => Ok(file_remove_(idu, temp_file).await),
        DataFileParam::Download(file_key, file_name) => {
            Ok(file_download_init(idu, file_key, file_name).await)
        }
        DataFileParam::Letter(idb) => Ok(file_letter(idu, idb).await),
        DataFileParam::Forward(idb) => Ok(file_forward(idu, idb).await),
    }
}

async fn file_download_init(idu: i32, file_key: String, file_name: String) -> DataResults {
    let user = db_user_by_idu(idu).await;
    let path_to_file = path_to_attachment(&user.email, &file_key);
    download_file(&path_to_file, &file_name, "Файл не найден").await
}

async fn file_remove_(idu: i32, temp_file: String) -> DataResults {
    if file_remove(&attach_temp_file(idu, &temp_file)).await {
        DataResults::Files(DataFileResults::Remove(temp_file))
    } else {
        DataResults::Error("Ошибка при удалении файла".to_string())
    }
}

async fn file_upload(idu: i32, list: FileUpload) -> DataResults {
    let mut results: Vec<DataFileItem> = Vec::new();
    for (file_name, file_data) in list {
        push_uploaded(idu, file_name, file_data, &mut results).await;
    }
    DataResults::Files(DataFileResults::Uploaded(results))
}

async fn push_uploaded(
    idu: i32,
    file_name: String,
    file_data: Vec<u8>,
    results: &mut Vec<DataFileItem>,
) {
    let size = file_data.len() as u64;
    let temp_name = uuid::Uuid::new_v4().to_string();
    let temp_file = attach_temp_file(idu, &temp_name);
    if file_write(&temp_file, file_data).await {
        results.push(DataFileItem {
            type_: DataFileType::Temp(temp_name),
            file_name,
            size,
        });
    }
}

async fn file_letter(idu: i32, idb: i64) -> DataResults {
    let user = db_user_by_idu(idu).await;
    if let Some(file_name) = db_message_source(idu, idb).await {
        let path_to_file = path_to_source(&user.email, &file_name);
        download_file(&path_to_file, &file_name, "Ошибка[1] при поиске оригинала").await
    } else {
        DataResults::Error("Ошибка[2] при поиске оригинала".to_string())
    }
}

async fn download_file(path_to_file: &str, file_name: &str, error: &str) -> DataResults {
    if tokio::fs::metadata(path_to_file).await.is_ok()
        && let Some(data) = tokio::fs::read(path_to_file).await.ok()
    {
        DataResults::Files(DataFileResults::Download((file_name.to_string(), data)))
    } else {
        DataResults::Error(error.to_string())
    }
}

async fn file_forward(idu: i32, idb: i64) -> DataResults {
    if let Some(data) = db_message(idu, idb).await {
        let mut results: Vec<DataFileItem> = Vec::new();
        if let Some(attach) = data.attachments.clone() {
            let user = db_user_by_idu(idu).await;
            for item in attach.list.iter() {
                let path_to_file = path_to_attachment_k_id(&user.email, &attach.key, item.id);
                if let Ok(file_data) = tokio::fs::read(path_to_file).await {
                    push_uploaded(idu, item.file_name.clone(), file_data, &mut results).await;
                }
            }
        }
        DataResults::Files(DataFileResults::Uploaded(results))
    } else {
        DataResults::Skip
    }
}
