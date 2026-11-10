use crate::{
    db::{t_notes::db_trusted_domains, t_user::db_all_users},
    paths::TEMP_DIR,
    tasks::store::{TRUSTED_DOMAINS, USER_BY_EMAIL},
};
use std::process::Command;

pub async fn task_init() {
    if let Err(err) = tokio::fs::create_dir_all(TEMP_DIR).await {
        tracing::error!("test_dirs: {:?}", err);
    }

    for row in db_all_users().await {
        if let Ok(mut users) = USER_BY_EMAIL.lock() {
            users.insert(row.email.clone(), row.idu);
        }
        /*if let Ok(mut users) = USER_BY_ID.lock() {
            users.insert(
                row.idu,
                TUser {
                    idu: row.idu,
                    email: row.email.clone(),
                    name: row.name.clone(),
                },
            );
        }*/
    }

    let domains = db_trusted_domains().await;
    if let Ok(mut list) = TRUSTED_DOMAINS.lock() {
        list.extend(domains);
        list.extend(
            ["hh.ru", "gmail.com", "hotmail.com", "zoom.us", "evrone.net"]
                .iter()
                .map(|v| v.to_string()),
        );
    }
}

// --

pub async fn run_tasks() {
    let mut interval_timer = tokio::time::interval(chrono::Duration::hours(1).to_std().unwrap());
    loop {
        interval_timer.tick().await;
        tokio::spawn(async {
            // закрываем отвалившиеся соединения
            // sse_cleaner();

            // удаляем файлы из временной директории, которым более суток
            Command::new("sh")
                .arg("-c")
                .arg(format!("find {TEMP_DIR} -type f -ctime +1d -delete"))
                .output()
                .expect("failed to execute process");
        });
    }
}
