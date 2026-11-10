use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex},
};

/*
pub struct TUser {
    pub idu: i32,
    pub email: String,
    pub name: String,
}

pub static USER_BY_ID: LazyLock<Mutex<HashMap<i32, TUser>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
*/

pub static USER_BY_EMAIL: LazyLock<Mutex<HashMap<String, i32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static TRUSTED_DOMAINS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
