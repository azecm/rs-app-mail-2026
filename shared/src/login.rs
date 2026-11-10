use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default)]
pub struct DataLoginParams {
    pub login: String,
    pub pass: String,
    pub email: String,
}
