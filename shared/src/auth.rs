use bitcode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone, Default)]
pub struct DataAuthParams {
    pub session_key: String,
}
