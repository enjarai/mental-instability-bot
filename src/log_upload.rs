use std::{
    borrow::Cow,
    io::{Cursor, ErrorKind, Read},
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{Attachment, Message},
    builder::{CreateActionRow, CreateButton, CreateEmbed},
};

use serenity::client::Context;

use crate::{
    constants::{MAX_LOG_SIZE, MCLOGS_API_BASE_URL, PASTEBIN_URL, PASTE_GG_API_BASE_URL}, log_checking::{check_logs, environment::read_mc_version}, mappings::cache::MappingsCache, util::format_bytes, ConfigData, MappingsCacheKey
};

#[derive(Deserialize, Clone)]
struct UploadData {
    url: Option<String>,
    #[allow(dead_code)]
    error: Option<String>,
}

#[derive(Serialize)]
struct LogUpload<'a> {
    content: &'a str,
}

type Log = (String, LogType, MapStatus, String, String);

pub(crate) enum LogType {
    Uploaded,
    Reuploaded,
    Downloaded,
}

pub(crate) enum MapStatus {
    InvalidMcVersion,
    Unmapped,
    Mapped(Duration),
    NotRequired,
}

impl LogType {
    pub(crate) fn title_format(&self, name: &str, took: &Duration) -> String {
        match self {
            Self::Uploaded => format!("Uploaded {name} in {}ms", took.as_millis()),
            Self::Reuploaded => format!("Reuploaded {name} in {}ms", took.as_millis()),
            Self::Downloaded => format!("Scanned {name} in {}ms", took.as_millis()),
        }
    }
}

pub(crate) async fn check_for_logs(
    ctx: &Context,
    message: &Message,
    all: bool,
) -> Result<Option<(&'static str, Vec<CreateEmbed>, Vec<CreateActionRow>)>> {
    let mut data = ctx.data.write().await;
    let attachments: Vec<_> =
        if let Some(file_extensions) = &data.get::<ConfigData>().unwrap().log_extensions {
            message
                .attachments
                .iter()
                .filter(|attachment| all || is_valid_log(attachment, file_extensions))
                .collect()
        } else {
            vec![]
        };

    let mappings_cache = data.get_mut::<MappingsCacheKey>().unwrap();

    let mut logs: Vec<Log> = upload_log_files(mappings_cache, &attachments).await?;
    logs.append(&mut check_pre_uploaded_logs(mappings_cache, &message.content).await?);

    if logs.is_empty() {
        return Ok(None);
    }

    let edit = (
        "",
        logs.iter()
            .map(|(name, t, m, _, log)| check_logs(log, name, t, m))
            .collect(),
        vec![CreateActionRow::Buttons(
            logs.iter()
                .map(|(name, _, _, url, _)| CreateButton::new_link(url).label(name))
                .collect(),
        )],
    );

    Ok(Some(edit))
}

fn is_valid_log<T: AsRef<str>>(attachment: &Attachment, allowed_extensions: &[T]) -> bool {
    attachment.size < 1_000_000
        && (allowed_extensions
            .iter()
            .any(|extension| attachment.filename.ends_with(extension.as_ref())))
}

async fn try_remap<'a>(
    mappings_cache: &mut MappingsCache,
    log: &'a str,
) -> Result<(Cow<'a, str>, MapStatus)> {
    let mut map_status = MapStatus::Unmapped;

    if let Some(mc_version) = read_mc_version(log) {
        map_status = MapStatus::InvalidMcVersion;

        let start = Instant::now();

        if let Some(mappings) = mappings_cache.get_or_download(&mc_version).await? {
            let log = Cow::Owned(mappings.remap_log(log));
            map_status = MapStatus::Mapped(Instant::now() - start);

            return Ok((log, map_status));
        }
    }

    Ok((Cow::Borrowed(log), map_status))
}

async fn upload_log_files(
    mappings_cache: &mut MappingsCache,
    attachments: &[&Attachment],
) -> Result<Vec<Log>> {
    let mut responses = vec![];

    for attachment in attachments {
        if attachment.size > MAX_LOG_SIZE {
            return Err(anyhow!(
                "Log size of {} exceeds the maximum allowed size of {}",
                format_bytes(attachment.size),
                format_bytes(MAX_LOG_SIZE)
            ));
        }

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

        // Potentially perhaps remap some logs
        let (log, map_status) = try_remap(mappings_cache, &log).await?;

        let data = upload(&log).await?;

        if let Some(url) = data.url {
            responses.push((
                attachment.filename.clone(),
                LogType::Uploaded,
                map_status,
                url,
                log.into_owned(),
            ));
        } else {
            return Err(anyhow!(
                "Mclo.gs uploading error: {}",
                data.error.unwrap_or("Unknown error".to_string())
            ));
        }
    }

    Ok(responses)
}

async fn check_pre_uploaded_logs(
    mappings_cache: &mut MappingsCache,
    message_content: &str,
) -> Result<Vec<Log>> {
    let mut logs: Vec<_> = vec![];

    for (url, id) in find_urls(r"https:\/\/mclo\.gs\/([a-zA-Z0-9]+)", message_content) {
        let log_data = download(&id).await?;

        logs.push((
            id,
            LogType::Downloaded,
            MapStatus::NotRequired,
            Some(url),
            log_data,
        ));
    }

    for (_, id) in find_urls(
        r"https:\/\/paste\.gg\/p\/\w+\/([a-zA-Z0-9]+)",
        message_content,
    ) {
        if let Some(log_data) = download_paste_gg(&id).await? {
            logs.push((
                id,
                LogType::Downloaded,
                MapStatus::NotRequired,
                None,
                log_data,
            ));
        }
    }

    for (_, id) in find_urls(r"https:\/\/pastebin\.com\/([a-zA-Z0-9]+)", message_content) {
        let log_data = download_pastebin(&id).await?;

        logs.push((
            id,
            LogType::Downloaded,
            MapStatus::NotRequired,
            None,
            log_data,
        ));
    }

    let mut responses: Vec<Log> = vec![];

    for log in logs {
        let (remapped, map_status) = try_remap(mappings_cache, &log.4).await?;

        if *remapped == log.4
            && let Some(url) = log.3
        {
            responses.push((log.0, log.1, MapStatus::NotRequired, url, log.4));
        } else {
            let data = upload(&remapped).await?;
            responses.push((
                log.0,
                LogType::Reuploaded,
                map_status,
                data.url.ok_or(anyhow!("Couldn't reupload"))?,
                remapped.into_owned(),
            ));
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
        .json::<UploadData>()
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

async fn download_pastebin(id: &str) -> Result<String> {
    let client: reqwest::Client = reqwest::Client::new();

    let response = client
        .get(format!("{PASTEBIN_URL}/raw/{id}"))
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}
