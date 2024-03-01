use std::{
    collections::HashMap, io::{Cursor, ErrorKind, Read}
};

use anyhow::Result;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{Attachment, Message},
    builder::{CreateActionRow, CreateButton, CreateEmbed, EditMessage},
};

use serenity::client::Context;

use crate::{constants::MCLOGS_BASE_URL, get_config, log_checking::{check_checks, CheckResult}};

#[derive(Deserialize, Clone)]
struct LogData {
    success: bool,
    url: Option<String>,
    _error: Option<String>,
}

#[derive(Serialize)]
struct LogUpload<'a> {
    content: &'a str,
}

pub(crate) async fn check_for_logs(ctx: &Context, message: &Message, all: bool) -> Result<usize> {
    if let Some(file_extensions) = &get_config!(ctx).log_extensions {
        let attachments: Vec<_> = message
            .attachments
            .iter()
            .filter(|attachment| all || is_valid_log(attachment, &file_extensions))
            .collect();

        if attachments.is_empty() {
            return Ok(0);
        }

        let mut reply = message.reply(ctx, "Logs detected, uploading...").await?;
        let logs = upload_log_files(ctx, &attachments).await?;

        let edit = if logs.is_empty() {
            EditMessage::default().content("Failed to upload!")
        } else {
            EditMessage::default()
                .content("")
                .embeds(logs.iter()
                    .map(|(name, (_, check))| {
                        let mut embed = CreateEmbed::new()
                            .title(format!("Uploaded {}", name))
                            .color(check.severity.get_color());
        
                        for (title, body) in &check.reports {
                            embed = embed.field(title, body, true);
                        }
        
                        embed
                    })
                    .collect()
                )
                .components(vec![CreateActionRow::Buttons(
                    logs.iter()
                        .map(|(name, (log, _))| (name, log))
                        .filter(|(_, log)| log.url.is_some())
                        .map(|(name, log)| {
                            CreateButton::new_link(log.url.clone().unwrap()).label(name)
                        })
                        .collect(),
                )])
        };

        reply.edit(ctx, edit).await?;

        Ok(attachments.len())
    } else {
        Ok(0)
    }
}

fn is_valid_log<T: AsRef<str>>(attachment: &Attachment, allowed_extensions: &[T]) -> bool {
    attachment.size < 1_000_000
        && (allowed_extensions
            .iter()
            .any(|extension| attachment.filename.ends_with(extension.as_ref())))
}

async fn upload_log_files(ctx: &Context, attachments: &[&Attachment]) -> Result<HashMap<String, (LogData, CheckResult)>> {
    let mut responses = HashMap::new();

    for attachment in attachments {
        let data = if attachment.filename.ends_with(".gz") {
            let mut reader = GzDecoder::new(Cursor::new(
                attachment
                    .download()
                    .await
                    .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?,
            ));

            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            buf
        } else {
            attachment.download().await?
        };
        let log = String::from_utf8_lossy(&data);

        let data = upload(&log).await?;

        if data.success {
            responses.insert(attachment.filename.clone(), (data, check_checks(ctx, &log).await?));
        }
    }

    Ok(responses)
}

async fn upload(log: &str) -> Result<LogData> {
    let client = reqwest::Client::new();

    Ok(client
        .post(format!("{}/1/log", MCLOGS_BASE_URL))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(serde_urlencoded::to_string(LogUpload { content: log })?)
        .send()
        .await?
        .json()
        .await?)
}
