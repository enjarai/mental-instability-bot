use poise::CreateReply;
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use super::{Context, Error};

#[derive(Deserialize)]
struct ModVersion {
    version_number: String,
}

/// Get the current version of a mod for a given Minecraft version
#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub(crate) async fn modversion(
    ctx: Context<'_>,
    #[description = "The slug of the Modrinth project"] mut slug: String,
    #[description = "The Minecraft version to look for a version for"] version: Option<String>,
    #[description = "The modloader to look for a version for"] loader: Option<String>,
) -> Result<(), Error> {
    let mut loader = loader.unwrap_or("fabric".to_string());

    slug.retain(|c| char::is_ascii_alphanumeric(&c) || r"-_".contains(c));
    loader.retain(|c| char::is_ascii_alphabetic(&c));
    let loader = loader.to_lowercase();

    let version = version.map(|v| {
        let mut v2 = v.clone();
        v2.retain(|c| char::is_ascii_alphanumeric(&c) || r"-_+.".contains(c));
        v2
    });

    match get_mod_version(&slug, version.as_deref(), &loader).await? {
        Ok(version) => {
            ctx.send(CreateReply::default().content(format!(
                "Latest available version for {} ({}) is:\n`{}`",
                slug, loader, version
            )))
            .await?;
        }
        Err(err) => {
            ctx.send(CreateReply::default().content(err)).await?;
        }
    }

    Ok(())
}

pub async fn get_mod_version(
    slug: &str,
    mc_version: Option<&str>,
    loader: &str,
) -> Result<Result<String, String>, Error> {
    let client = Client::builder()
        .user_agent("enjarai/mental-instability-bot (enjarai@protonmail.com)")
        .build()?;

    let version_query = match mc_version {
        Some(version) => {
            format!("game_versions=[%22{version}%22]&")
        }
        None => String::new(),
    };

    match client
        .get(format!(
            "{}/project/{}/version?{}loaders=[%22{}%22]",
            crate::constants::MODRINTH_API_URL,
            slug,
            version_query,
            loader
        ))
        .send()
        .await?
        .error_for_status()
    {
        Ok(modrinth_response) => {
            let mut mod_versions: Vec<ModVersion> =
                serde_json::from_str(&modrinth_response.text().await?)?;

            if mod_versions.len() > 0 {
                let mod_version = mod_versions.remove(0);
                Ok(Ok(mod_version.version_number))
            } else {
                Ok(Err(format!(
                    "No valid versions found for:\n{} `{}` ({})",
                    slug,
                    mc_version.unwrap_or("any"),
                    loader
                )))
            }
        }
        Err(err) => {
            if err.status() == Some(StatusCode::NOT_FOUND) {
                Ok(Err(format!("Not a valid Modrinth project: {slug}")))
            } else {
                return Ok(Err(format!("Unknown status code returned by API: {err}")));
            }
        }
    }
}
