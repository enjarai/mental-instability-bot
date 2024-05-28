use poise::CreateReply;
use serde::Deserialize;

use super::{Context, Error};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct YarnVersion {
    game_version: Option<String>,
    version: Option<String>,
}

#[derive(Deserialize)]
struct LoaderWrapper {
    loader: Option<LoaderVersion>,
}

#[derive(Deserialize)]
struct LoaderVersion {
    version: Option<String>,
}

/// Get current Fabric versions for a given Minecraft version
#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub(crate) async fn version(
    ctx: Context<'_>,
    #[description = "Get the latest Fabric versions for the given game version"] version: String,
) -> Result<(), Error> {
    let mut yarn_response: &str = &reqwest::get(format!(
        "{}/versions/yarn/{}?limit=1",
        crate::constants::FABRIC_META_URL,
        version
    ))
    .await?
    .text()
    .await?;
    if let Some(stripped) = yarn_response.strip_prefix('[') {
        yarn_response = stripped;
    }
    if let Some(stripped) = yarn_response.strip_suffix(']') {
        yarn_response = stripped;
    }
    if yarn_response.trim().is_empty() {
        yarn_response = "{}"
    }
    let yarn_version: YarnVersion = serde_json::from_str(yarn_response)?;

    let mut loader_response: &str = &reqwest::get(format!(
        "{}/versions/loader/{}?limit=1",
        crate::constants::FABRIC_META_URL,
        version
    ))
    .await?
    .text()
    .await?;
    if let Some(stripped) = loader_response.strip_prefix('[') {
        loader_response = stripped;
    }
    if let Some(stripped) = loader_response.strip_suffix(']') {
        loader_response = stripped;
    }
    if loader_response.trim().is_empty() {
        loader_response = "{}"
    }
    let loader_version: LoaderWrapper = serde_json::from_str(loader_response)?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "```
minecraft_version={}
yarn_mappings={}
loader_version={}```",
                yarn_version
                    .game_version
                    .unwrap_or_else(|| "unknown".to_owned()),
                yarn_version.version.unwrap_or_else(|| "unknown".to_owned()),
                loader_version
                    .loader
                    .and_then(|loader| loader.version)
                    .unwrap_or_else(|| "unknown".to_owned())
            ))
    )
    .await?;

    Ok(())
}
