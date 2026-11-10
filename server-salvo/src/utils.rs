use bytes::Bytes;
use shared::{DataEBox, DataResults, TEST_KEY, encrypt_data};

pub fn data_to_bytes(data: &DataResults) -> Bytes {
    encrypt_data(data, [0; 32], None, &TEST_KEY)
        .map(Bytes::from)
        .unwrap_or_else(|err| {
            tracing::error!("data_to_bytes {}", err);
            Bytes::new()
        })
}

pub fn ebox_box(ebox: DataEBox) -> i32 {
    match ebox {
        DataEBox::Incoming => 0,
        DataEBox::Saved => 1,
        DataEBox::Sent => 2,
        DataEBox::Deleted => 3,
    }
}
