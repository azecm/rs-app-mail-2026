use crate::notes::DataNoteResults;
use crate::user::DataUser;
use bitcode::{Decode, Encode};
use crate::DataMessageBoxed;

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct DataInitResults {
    pub notes: Vec<DataNoteResults>,
    pub user: DataUser,
    pub news: Vec<DataMessageBoxed>,
}
