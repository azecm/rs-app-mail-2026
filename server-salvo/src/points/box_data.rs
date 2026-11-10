use shared::{DataBoxParams, DataBoxResults, DataMessageBoxed, DataResults};

use crate::{
    db::t_boxes::{DBBox, db_box_page},
    points::common::transport_messages,
};

pub async fn data_box(idu: i32, data: DataBoxParams) -> anyhow::Result<DataResults> {
    Ok(DataResults::Box(DataBoxResults {
        page: data.page,
        ebox: data.ebox,
        data: box_messages(db_box_page(idu, data.ebox, data.page).await),
    }))
}

pub fn box_messages(list: Vec<DBBox>) -> Vec<DataMessageBoxed> {
    list.iter().map(transport_messages).collect()
}
