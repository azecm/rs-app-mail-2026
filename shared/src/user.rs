use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Default, Clone)]
pub struct DataUser {
    pub email: String,
    pub prefix: String,
    pub signature: String,
}
