use crate::{
    channels::send_by_idu,
    db::{
        t_boxes::{
            DBMailAddress, DBMailAttachmentItem, DBMailAttachments, db_message, db_message_add,
            db_message_move,
        },
        t_user::db_user_by_idu,
    },
    mail_send::send_message,
    paths::{attach_temp_file, file_rename, path_to_attachment_k_id},
    points::common::transport_messages,
};
use shared::{
    DataEBox, DataFileItem, DataMessageParams, DataMessageResults, DataResults, SSEMessage,
    ServerEvent,
};
use uuid::Uuid;

pub async fn data_messages(idu: i32, data: DataMessageParams) -> anyhow::Result<DataResults> {
    let results: DataResults = match data {
        DataMessageParams::Move(idb, box_from, box_to) => {
            move_message(idu, idb, box_from, box_to).await
        }
        DataMessageParams::Load(idb, ebox) => load_message(idu, idb, ebox).await,
        DataMessageParams::Send(recipient, subject, content, attachment) => {
            send_email(recipient, subject, content, attachment, idu).await
        }
    };
    Ok(results)
}

async fn move_message(idu: i32, idb: i64, ebox_from: DataEBox, ebox_to: DataEBox) -> DataResults {
    if db_message_move(idu, idb, ebox_to).await {
        send_by_idu(
            idu,
            ServerEvent::Message(SSEMessage::Moved(idb, ebox_from, ebox_to)),
        )
        .await;
        DataResults::Skip
    } else {
        DataResults::Error("Ошибка при перемещении".to_string())
    }
}

async fn load_message(idu: i32, idb: i64, ebox: DataEBox) -> DataResults {
    if let Some(data) = db_message(idu, idb).await {
        DataResults::Message(DataMessageResults::Loaded(transport_messages(&data), ebox))
    } else {
        DataResults::Skip
    }
}

async fn send_email(
    recipient: String,
    subject: String,
    content: String,
    attachments: Vec<DataFileItem>,
    idu: i32,
) -> DataResults {
    let user = db_user_by_idu(idu).await;
    let sender = format!("{} <{}>", user.name, user.email);
    let send_result =
        send_message(&sender, &recipient, &subject, &content, &attachments, idu).await;

    if send_result {
        let (name, address) = email_struct(&sender);
        let sender: DBMailAddress = DBMailAddress { name, address };

        let (name, address) = email_struct(&recipient);
        let recipient: DBMailAddress = DBMailAddress { name, address };

        let mut res_attachments: Option<DBMailAttachments> = None;
        if !attachments.is_empty() {
            let key = Uuid::new_v4().to_string();
            let mut list: Vec<DBMailAttachmentItem> = Vec::new();
            for (index, item) in attachments.iter().enumerate() {
                let id = index + 1;
                let target_file = path_to_attachment_k_id(&sender.address, &key, id);
                let source_file = attach_temp_file(idu, &item.temp_file());

                if file_rename(&source_file, &target_file).await {
                    list.push(DBMailAttachmentItem {
                        id,
                        size: item.size,
                        file_name: item.file_name.to_string(),
                    });
                }
            }
            res_attachments = Some(DBMailAttachments { key, list })
        }

        db_message_add(
            idu,
            DataEBox::Sent,
            String::new(),
            sender,
            recipient,
            subject,
            content,
            res_attachments,
        )
        .await;
    }
    DataResults::Message(DataMessageResults::Send(send_result))
}

fn email_struct(text: &str) -> (Option<String>, String) {
    if let Ok(data) = mailparse::addrparse(text)
        && let Some(email) = data.first()
    {
        match email {
            mailparse::MailAddr::Group(info) => {
                if let Some(info) = info.addrs.first() {
                    return (info.display_name.clone(), info.addr.clone());
                }
            }
            mailparse::MailAddr::Single(info) => {
                return (info.display_name.clone(), info.addr.clone());
            }
        }
    }
    (None, "".to_string())
}
