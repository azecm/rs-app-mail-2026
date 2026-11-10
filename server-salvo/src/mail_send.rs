use crate::paths::attach_temp_file;
use lettre::message::{MultiPart, SinglePart, header};
use lettre::{Message, SendmailTransport, Transport};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::fs;

use shared::DataFileItem;

pub async fn send_message(
    sender: &str,
    recipient: &str,
    subject: &str,
    message_md: &str,
    attachments: &[DataFileItem],
    idu: i32,
) -> bool {
    let mut result = false;

    let parser = pulldown_cmark::Parser::new(message_md);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    let text_output = markdown_to_plain_text(message_md);

    let mut multipart = MultiPart::alternative()
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(text_output),
        )
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(html_output.to_string()),
        );

    if !attachments.is_empty() {
        multipart = MultiPart::mixed().multipart(multipart);
        for item in attachments.iter() {
            let path_to_file = attach_temp_file(idu, &item.temp_file());
            let mime = mime_guess::from_path(&item.file_name).first_or_octet_stream();
            if let Ok(content_type) = header::ContentType::parse(mime.as_ref())
                && let Ok(f) = fs::read(path_to_file)
            {
                multipart = multipart.singlepart(
                    SinglePart::builder()
                        .header(content_type)
                        .header(header::ContentDisposition::attachment(&item.file_name))
                        .header(header::ContentTransferEncoding::Base64)
                        .body(f),
                );
            }
        }
    }

    if let Ok(sender) = sender.parse()
        && let Ok(recipient) = recipient.parse()
    {
        let message = Message::builder()
            .from(sender)
            .to(recipient)
            .subject(subject)
            .multipart(multipart);

        match message {
            Ok(email) => {
                let mailer = SendmailTransport::new();
                match mailer.send(&email) {
                    Ok(_) => {
                        result = true;
                    }
                    Err(err) => {
                        tracing::error!("send_message[1]: {:?}", err);
                    }
                }
            }
            Err(err) => {
                tracing::warn!("send_message[2] {:?}", err);
            }
        }
    }

    result
}

fn markdown_to_plain_text(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut plain_text = String::new();

    for event in parser {
        match event {
            Event::Text(text) => {
                plain_text.push_str(&text);
            }
            Event::SoftBreak => {
                plain_text.push('\n');
            }
            Event::HardBreak => {
                plain_text.push('\n');
            }
            Event::Rule => {
                plain_text.push('\n');
                plain_text.push('-');
                plain_text.push('-');
                plain_text.push('-');
                plain_text.push('-');
            }
            Event::End(TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::List(_) | TagEnd::Item) => {
                plain_text.push('\n');
            }
            Event::Start(Tag::Paragraph | Tag::Heading { .. } | Tag::List(_) | Tag::Item) => {
                plain_text.push('\n');
            }
            _ => {}
        }
    }
    plain_text
}
