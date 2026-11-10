use bitcode::{Decode, Encode};

pub type FileUpload = Vec<(String, Vec<u8>)>;

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataFileParam {
    Upload(FileUpload),
    Remove(String),
    Download(String, String),
    Letter(i64),
    Forward(i64),
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataFileType {
    Temp(String),
    Index(usize),
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct DataFileItem {
    pub type_: DataFileType,
    pub file_name: String,
    pub size: u64,
}
impl DataFileItem {
    pub fn temp_file(&self) -> String {
        match self.type_.clone() {
            DataFileType::Temp(name) => name,
            _ => String::new(),
        }
    }
    pub fn saved_file(&self, k: &str) -> String {
        match self.type_.clone() {
            DataFileType::Index(ind) => format!("{k}-{ind}"),
            _ => String::new(),
        }
    }
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum DataFileResults {
    Uploaded(Vec<DataFileItem>),
    Remove(String),
    Download((String, Vec<u8>)),
}
