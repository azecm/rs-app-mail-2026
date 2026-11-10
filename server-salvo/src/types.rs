use serde::{Deserialize, Serialize};

/*
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DownloadStruct {
    pub path: String,
    pub name: String,
}
*/

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub idu: i32,
    pub browser: String,
    pub device: [u8; 32],
    pub active: bool,
}
impl Session {
    pub fn new(idu: i32, browser: String, device: [u8; 32]) -> Self {
        Self {
            idu,
            browser,
            device,
            active: false,
        }
    }

    pub fn active(&self) -> Self {
        Self {
            active: true,
            ..self.clone()
        }
    }

    pub fn auth(&self) -> Self {
        self.clone()
    }
}
