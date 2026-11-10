use crate::db::t_boxes::DBBox;
use shared::{
    DataAttachments, DataFileItem, DataFileType, DataMailAddress, DataMessageBoxed, DataMessageSrc,
};

pub fn transport_messages(r: &DBBox) -> DataMessageBoxed {
    Box::new(DataMessageSrc {
        idb: r.idb,
        content: r.content.clone(),
        subject: r.subject.clone(),
        order: r.order,
        has_source: !r.source_file.is_empty(),
        date: r.date.clone(),
        sender: DataMailAddress {
            address: r.sender.address.clone(),
            name: r.sender.name.clone(),
        },
        recipient: DataMailAddress {
            address: r.recipient.address.clone(),
            name: r.recipient.name.clone(),
        },
        attachments: r.attachments.clone().map(|r| DataAttachments {
            key: r.key.clone(),
            list: r
                .list
                .iter()
                .map(|r| DataFileItem {
                    type_: DataFileType::Index(r.id),
                    size: r.size,
                    file_name: r.file_name.clone(),
                })
                .collect::<Vec<_>>(),
        }),
    })
}
