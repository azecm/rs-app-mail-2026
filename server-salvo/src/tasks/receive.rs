use crate::db::t_boxes::{DBMailAddress, DBMailAttachmentItem, DBMailAttachments, db_message_add};
use crate::paths::{path_to_attachment_k_id, path_to_source};
use crate::tasks::store::TRUSTED_DOMAINS;
use crate::{paths::MAIL_SOURCE_PATH, tasks::store::USER_BY_EMAIL};
use lol_html::{HtmlRewriter, Settings, comments, element, html_content::ContentType};
use mail_parser::PartType::Binary;
use mail_parser::{Address, Message, MessageParser, MimeHeaders};
use shared::DataEBox;
use std::sync::LazyLock;
use tokio::fs;
use uuid::Uuid;

static TRUSTED: LazyLock<Vec<String>> = LazyLock::new(|| {
    [
        ".ru",
        ".dev",
        ".org",
        ".net",
        "gmail.com",
        "hotmail.com",
        "zoom.us",
    ]
    .iter()
    .map(|v| v.to_string())
    .collect::<Vec<String>>()
});

static DISABLED_FROM: LazyLock<Vec<String>> = LazyLock::new(|| {
    ["тинькоф", "тинкоф", "tinkof"]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
});

// https://crates.io/crates/notify

pub async fn mail_watcher() {
    let mut interval_timer = tokio::time::interval(chrono::Duration::seconds(5).to_std().unwrap());
    loop {
        interval_timer.tick().await;
        for path_dir in watch_dirs() {
            match fs::metadata(&path_dir).await {
                Ok(_) => match fs::read_dir(&path_dir).await {
                    Ok(mut entries) => {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            if let Some(path_to_file) = entry.path().to_str() {
                                read_email(path_to_file).await;
                            }
                        }
                    }
                    Err(err) => tracing::error!("watch_dirs[1]: {path_dir} -- {err}"),
                },
                Err(err) => tracing::error!("watch_dirs[2]: {path_dir} -- {err}"),
            }
        }
    }
}

fn watch_dirs() -> Vec<String> {
    let emails = USER_BY_EMAIL
        .lock()
        .ok()
        .map(|users| users.keys().cloned().collect::<Vec<String>>())
        .unwrap_or_default();

    emails
        .iter()
        .map(|email| {
            let list = email.split('@').collect::<Vec<_>>();
            let domain = list[1];
            let user = list[0];
            format!("{MAIL_SOURCE_PATH}/{domain}/{user}/new")
        })
        .collect::<Vec<_>>()
}

fn get_email_from_path(path: &str) -> String {
    let list = path.split('/').collect::<Vec<_>>();
    format!("{}@{}", list[list.len() - 3], list[list.len() - 4])
}

async fn read_email(path_to_file: &str) {
    if get_file_name(path_to_file).starts_with('.') {
        return;
    }
    match fs::metadata(path_to_file).await {
        Ok(_) => {}
        Err(_) => {
            return;
        }
    }
    let current_email = get_email_from_path(path_to_file);

    match fs::read(path_to_file).await {
        Ok(data) => {
            if let Some(message) = MessageParser::default().parse(&data) {
                prepare(message, &current_email, path_to_file).await;
            }
        }
        Err(err) => {
            tracing::error!("read_email[1] {err:?}");
        }
    }

    //если не перемещен - удаляем
    if fs::metadata(path_to_file).await.is_ok() {
        let _ = fs::remove_file(path_to_file).await.ok();
    }
}

async fn prepare<'a>(message: Message<'a>, current_email: &str, path_to_file: &str) {
    let from = message.from();
    let to = message.to();
    let subject = message.subject().unwrap_or_default().to_string();
    let html = match message.body_html(0) {
        Some(html) => html.to_string(),
        None => "".to_string(),
    };
    let text = match message.body_text(0) {
        Some(text) => text.to_string(),
        None => "".to_string(),
    };

    let content = if !html.is_empty() {
        htmd::convert(&clean_html(&html)).ok().unwrap_or_default()
    } else if !text.is_empty() {
        text
    } else {
        "".to_string()
    };

    let key = Uuid::new_v4().to_string();
    let mut list: Vec<DBMailAttachmentItem> = vec![];
    for part in message.attachments() {
        if part.is_binary()
            && let Some(file_name) = part.attachment_name()
            && let Binary(body) = &part.body
        {
            let id = list.len() + 1;
            let size = body.len() as u64;

            let file_path = path_to_attachment_k_id(current_email, &key, id);
            match fs::create_dir_all(get_dir_path(&file_path)).await {
                Ok(_) => match fs::write(&file_path, body).await {
                    Ok(_) => {
                        list.push(DBMailAttachmentItem {
                            id,
                            size,
                            file_name: file_name.to_string(),
                        });
                    }
                    Err(err) => {
                        tracing::error!("save attachment: {:?}", err);
                    }
                },
                Err(err) => {
                    tracing::error!("save attachment: {:?}", err);
                }
            }
        }
    }

    let attachments: Option<DBMailAttachments> = if list.is_empty() {
        None
    } else {
        Some(DBMailAttachments { key, list })
    };

    let sender = mail_address_from_header(from);
    let recipient = mail_address_from_header(to);

    // ==

    let mut flag_spam = !format!("{:?}", to).to_lowercase().contains(current_email);

    let flag_name = sender.name.clone().map(|text| {
        let text = text.to_lowercase();
        DISABLED_FROM.iter().any(|t| text.contains(t))
    });
    if flag_name == Some(true) {
        flag_spam = true;
    }

    for ends in TRUSTED.iter() {
        if sender.address.ends_with(ends) {
            flag_spam = false;
        }
    }

    let mut enabled_domain = false;
    if let Some(sender_domain) = email_host(&sender.address)
        && let Ok(list) = TRUSTED_DOMAINS.lock()
    {
        enabled_domain = list.contains(&sender_domain);
    }

    if !enabled_domain || flag_spam {
        //println!("enabled_domain {:?}", sender);
        //println!("enabled_domain {enabled_domain}");
        //println!("flag_spam: {}", flag_spam);
    }

    let idu = match USER_BY_EMAIL.lock() {
        Ok(users) => match users.get(current_email) {
            Some(val) => *val,
            None => {
                return;
            }
        },
        Err(err) => {
            tracing::error!("db_box_insert[1]: {:?}", err);
            return;
        }
    };

    let source_file = uuid::Uuid::new_v4().to_string();
    let ebox_target = if flag_spam {
        DataEBox::Deleted
    } else {
        DataEBox::Incoming
    };

    // перемещаем прочтенные
    match fs::create_dir_all(path_to_source(current_email, "")).await {
        Ok(_) => match fs::read(path_to_file).await {
            Ok(data) => {
                if let Err(err) = fs::write(path_to_source(current_email, &source_file), data).await
                {
                    tracing::error!("move::source::write - {:?}", err);
                }
            }
            Err(err) => {
                tracing::error!("move::source::read - {:?}", err);
            }
        },
        Err(err) => {
            tracing::error!("move::source::dirs - {:?}", err);
        }
    }

    db_message_add(
        idu,
        ebox_target,
        source_file,
        sender,
        recipient,
        subject,
        content,
        attachments,
    )
    .await;
}

fn mail_address_from_header<'a>(header: Option<&Address<'a>>) -> DBMailAddress {
    if let Some(addr) = header.and_then(|data| data.first())
        && let Some(address) = addr.address()
    {
        DBMailAddress {
            address: address.to_string(),
            name: addr.name().map(|name| name.to_string()),
        }
    } else {
        DBMailAddress::default()
    }
}

fn get_dir_path(file_path: &str) -> String {
    if let Some(pos) = file_path.rfind('/') {
        file_path[..pos].to_string()
    } else {
        "".to_string()
    }
}

fn email_host(src: &str) -> Option<String> {
    src.split('@')
        .map(|v| v.to_lowercase())
        .collect::<Vec<_>>()
        .get(1)
        .map(|v| v.to_string())
}

fn get_file_name(file_path: &str) -> String {
    if let Some(pos) = file_path.rfind('/') {
        file_path[pos + 1..].to_string()
    } else {
        "".to_string()
    }
}

fn clean_html(source: &str) -> String {
    match clean_html_(source) {
        Ok(str) => str,
        Err(err) => {
            tracing::error!("clean_html: {:?}", err);
            String::new()
        }
    }
}

fn clean_html_(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = vec![];

    let mut rewriter = HtmlRewriter::new(
        Settings::new()
            .append_element_content_handler(element!("script,style,meta,base,link,title", |el| {
                el.remove();
                Ok(())
            }))
            .append_element_content_handler(element!("img", |el| {
                if let Some(alt) = el.get_attribute("alt") {
                    if alt.trim().is_empty() {
                        el.remove();
                    } else {
                        el.replace(&format!("[{alt}]"), ContentType::Text);
                    }
                } else {
                    el.remove();
                }
                Ok(())
            }))
            .append_element_content_handler(element!("[style]", |el| {
                el.remove_attribute("style");
                Ok(())
            }))
            .append_element_content_handler(element!("[bgcolor]", |el| {
                el.remove_attribute("bgcolor");
                Ok(())
            }))
            .append_element_content_handler(element!("[background]", |el| {
                el.remove_attribute("background");
                Ok(())
            }))
            .append_element_content_handler(element!("[bordercolor]", |el| {
                el.remove_attribute("bordercolor");
                Ok(())
            }))
            .append_element_content_handler(element!("[class]", |el| {
                el.remove_attribute("class");
                Ok(())
            }))
            .append_element_content_handler(element!("[color]", |el| {
                el.remove_attribute("color");
                Ok(())
            }))
            .append_element_content_handler(comments!("*", |c| {
                c.remove();
                Ok(())
            }))
            .append_element_content_handler(element!("a", |el| {
                el.prepend("*", ContentType::Text);
                Ok(())
            })),
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(source.as_bytes())?;
    rewriter.end()?;

    let html = String::from_utf8(output)?;
    Ok(html)
}
