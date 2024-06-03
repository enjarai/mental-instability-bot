use std::{
    io::{Cursor, ErrorKind, Read},
    path::Path,
    time::Duration,
};

use anyhow::Result;
use flate2::read::GzDecoder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{Attachment, Message},
    builder::{CreateActionRow, CreateButton, CreateEmbed},
};

use serenity::client::Context;

use crate::{
    constants::{MCLOGS_API_BASE_URL, PASTE_GG_API_BASE_URL},
    get_config,
    log_checking::check_logs,
};

#[derive(Deserialize, Clone)]
struct UploadData {
    url: Option<String>,
    _error: Option<String>,
}

#[derive(Serialize)]
struct LogUpload<'a> {
    content: &'a str,
}

type Log = (String, LogType, String, String);

pub(crate) enum LogType {
    Uploaded,
    Downloaded,
}

impl LogType {
    pub(crate) fn title_format(&self, name: &str, took: &Duration) -> String {
        match self {
            Self::Uploaded => format!("Uploaded {name} in {}ms", took.as_millis()),
            Self::Downloaded => format!("Scanned {name} in {}ms", took.as_millis()),
        }
    }
}

pub(crate) async fn check_for_logs(
    ctx: &Context,
    message: &Message,
    all: bool,
) -> Result<Option<(&'static str, Vec<CreateEmbed>, Vec<CreateActionRow>)>> {
    if let Some(file_extensions) = &get_config!(ctx).log_extensions {
        let attachments: Vec<_> = message
            .attachments
            .iter()
            .filter(|attachment| all || is_valid_log(attachment, file_extensions))
            .collect();

        let mut logs: Vec<Log> = upload_log_files(&attachments).await?;
        logs.append(&mut check_pre_uploaded_logs(&message.content).await?);

        if logs.is_empty() {
            return Ok(None);
        }

        let edit = (
            "",
            logs.iter()
                .map(|(name, t, _, log)| check_logs(log, name, t))
                .collect(),
            vec![CreateActionRow::Buttons(
                logs.iter()
                    .map(|(name, _, url, _)| CreateButton::new_link(url).label(name))
                    .collect(),
            )],
        );

        Ok(Some(edit))
    } else {
        Ok(None)
    }
}

fn is_valid_log<T: AsRef<str>>(attachment: &Attachment, allowed_extensions: &[T]) -> bool {
    attachment.size < 1_000_000
        && (allowed_extensions
            .iter()
            .any(|extension| attachment.filename.ends_with(extension.as_ref())))
}

async fn upload_log_files(attachments: &[&Attachment]) -> Result<Vec<Log>> {
    let mut responses = vec![];

    for attachment in attachments {
        let data = if Path::new(&attachment.filename)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("gz"))
        {
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

        if let Some(url) = data.url {
            responses.push((
                attachment.filename.clone(),
                LogType::Uploaded,
                url,
                log.into_owned(),
            ));
        }
    }

    Ok(responses)
}

async fn check_pre_uploaded_logs(message_content: &str) -> Result<Vec<Log>> {
    let mut responses = vec![];

    for (url, id) in find_urls(r"https:\/\/mclo\.gs\/([a-zA-Z0-9]+)", message_content) {
        let log_data = download(&id).await?;
        responses.push((id, LogType::Downloaded, url, log_data));
    }

    for (url, id) in find_urls(
        r"https:\/\/paste\.gg\/p\/\w+\/([a-zA-Z0-9]+)",
        message_content,
    ) {
        if let Some(log_data) = download_paste_gg(&id).await? {
            responses.push((id, LogType::Downloaded, url, log_data));
        }
    }

    Ok(responses)
}

fn find_urls(regex: &str, message_content: &str) -> Vec<(String, String)> {
    let regex = Regex::new(regex).unwrap();

    regex
        .captures_iter(message_content)
        .map(|caps| {
            (
                caps.get(0).expect("Regex err").as_str().to_string(),
                caps.get(1).expect("Regex err").as_str().to_string(),
            )
        })
        .collect()
}

async fn upload(log: &str) -> Result<UploadData> {
    let client = reqwest::Client::new();

    Ok(client
        .post(format!("{MCLOGS_API_BASE_URL}/1/log"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(serde_urlencoded::to_string(LogUpload { content: log })?)
        .send()
        .await?
        .json()
        .await?)
}

async fn download(id: &str) -> Result<String> {
    let client = reqwest::Client::new();

    Ok(client
        .get(format!("{MCLOGS_API_BASE_URL}/1/raw/{id}"))
        .send()
        .await?
        .text()
        .await?)
}

#[derive(Deserialize)]
struct GGResponse {
    result: GGResult,
}

#[derive(Deserialize)]
struct GGResult {
    files: Vec<GGFile>,
}

#[derive(Deserialize)]
struct GGFile {
    content: GGContent,
}

#[derive(Deserialize)]
struct GGContent {
    value: String,
}

async fn download_paste_gg(id: &str) -> Result<Option<String>> {
    let client: reqwest::Client = reqwest::Client::new();

    let mut response = client
        .get(format!("{PASTE_GG_API_BASE_URL}/pastes/{id}?full=true"))
        .send()
        .await?
        .json::<GGResponse>()
        .await?;

    if response.result.files.len() == 0 {
        return Ok(None);
    }

    Ok(Some(response.result.files.remove(0).content.value))
}
