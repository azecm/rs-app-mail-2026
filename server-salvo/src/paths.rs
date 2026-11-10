use const_format::concatcp;

#[cfg(target_os = "linux")]
const MAIL_ROOT_PATH: &str = "/home/user/_temp/mail/";
#[cfg(target_os = "macos")]
const MAIL_ROOT_PATH: &str = "/Users/mac-user/Documents/temp/mail/";
#[cfg(target_os = "freebsd")]
const MAIL_ROOT_PATH: &str = "/mnt/data/cache/mail/";

#[cfg(target_os = "linux")]
pub const MAIL_SOURCE_PATH: &str = "/home/user/_temp/mail/source";
#[cfg(target_os = "macos")]
pub const MAIL_SOURCE_PATH: &str = "/Users/mac-user/Documents/_cache/mail/virtual";
#[cfg(target_os = "freebsd")]
pub const MAIL_SOURCE_PATH: &str = "/var/mail/vhosts";

const DIR_ATTACHMENT: &str = "attachment";
const DIR_SOURCE: &str = "source";

pub const TEMP_EXT: &str = ".tmp";
pub const TXT_EXT: &str = ".txt";
pub const PATH_JOIN: &str = "/";

pub const TEMP_DIR: &str = concatcp!(MAIL_ROOT_PATH, "temp/");

pub fn path_to_attachment_k_id(email: &str, k: &str, id: usize) -> String {
    path_to_attachment(email, &format!("{k}-{id}"))
}

pub fn path_to_attachment(email: &str, base: &str) -> String {
    // base = {key}-{ind}
    format!("{MAIL_ROOT_PATH}{DIR_ATTACHMENT}/{email}/{base}{TEMP_EXT}")
}

pub fn path_to_source(email: &str, file_name: &str) -> String {
    format!(
        "{MAIL_ROOT_PATH}{DIR_SOURCE}/{email}/{file_name}{}",
        if !file_name.is_empty() {
            TXT_EXT
        } else {
            Default::default()
        }
    )
}

pub fn attach_temp_file(idu: i32, name: &str) -> String {
    TEMP_DIR.to_string() + &idu.to_string() + PATH_JOIN + name + TEMP_EXT
}

pub fn get_dir_path(file_path: &str) -> String {
    if let Some(pos) = file_path.rfind('/') {
        file_path[..pos].to_string()
    } else {
        "".to_string()
    }
}

pub async fn test_file_dir(file_path: &str) -> bool {
    if let Err(err) = tokio::fs::create_dir_all(get_dir_path(file_path)).await {
        tracing::error!("paths::test_file_dir {err}");
        false
    } else {
        true
    }
}

pub async fn file_write(file_path: &str, data: Vec<u8>) -> bool {
    if test_file_dir(file_path).await {
        match tokio::fs::write(file_path, data).await {
            Ok(_) => true,
            Err(err) => {
                tracing::error!("paths::file_write: {err}");
                false
            }
        }
    } else {
        false
    }
}

pub async fn file_remove(file_path: &str) -> bool {
    if test_file_dir(file_path).await {
        match tokio::fs::remove_file(file_path).await {
            Ok(_) => true,
            Err(err) => {
                tracing::error!("paths::file_remove: {err}");
                false
            }
        }
    } else {
        false
    }
}

pub async fn file_rename(source_file: &str, target_file: &str) -> bool {
    if test_file_dir(target_file).await {
        match tokio::fs::rename(&source_file, &target_file).await {
            Ok(_) => true,
            Err(err) => {
                tracing::error!("paths::file_rename: {err}");
                false
            }
        }
    } else {
        false
    }
}
