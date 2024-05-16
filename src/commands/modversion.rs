use poise::CreateReply;
use reqwest::StatusCode;
use serde::Deserialize;

use super::{Context, Error};

#[derive(Deserialize)]
struct ModVersion {
    version_number: String,
}

/// Get the current version of a mod for a given Minecraft version
#[poise::command(slash_command)]
pub(crate) async fn modversion(
    ctx: Context<'_>,
    #[description = "The slug of the Modrinth project"] mut slug: String,
    #[description = "The Minecraft version to look for a version for"] mut version: Option<String>,
    #[description = "The modloader to look for a version for"] loader: Option<String>,
) -> Result<(), Error> {
    let mut loader = loader.unwrap_or("fabric".to_string());

    slug.retain(|c| char::is_ascii_alphanumeric(&c) || r#"-_"#.contains(c));
    loader.retain(|c| char::is_ascii_alphabetic(&c));
    let loader = loader.to_lowercase();

    let version_query = match &mut version {
        Some(version) => {
            version.retain(|c| char::is_ascii_alphanumeric(&c) || r#"-_+."#.contains(c));
            format!("game_versions=[%22{}%22]&", version)
        },
        None => "".to_string()
    };

    match reqwest::get(format!(
        "{}/project/{}/version?{}loaders=[%22{}%22]",
        crate::constants::MODRINTH_API_URL,
        slug, version_query, loader
    )).await?.error_for_status() {
        Ok(modrinth_response) => {
            let mod_versions: Vec<ModVersion> = serde_json::from_str(&modrinth_response.text().await?)?;

            if let Some(mod_version) = mod_versions.get(0) {
                ctx.send(
                    CreateReply::default()
                    .content(format!(
                        "Latest available version for {} is:
`{}` ({})",
                        slug, mod_version.version_number, loader
                    ))
                ).await?;
            } else {
                ctx.send(
                    CreateReply::default()
                    .content(format!(
                        "No valid versions found for:
{} `{}` ({})",
                        slug, version.unwrap_or("any".to_string()), loader
                    ))
                ).await?;
            }
        },
        Err(err) => {
            if err.status() == Some(StatusCode::NOT_FOUND) {
                ctx.send(
                    CreateReply::default()
                    .content(format!(
                        "Not a valid Modrinth project: {}",
                        slug
                    ))
                ).await?;
            } else {
                return Err("Unknown status code returned by API".into());
            }
        }
    }

    Ok(())
}
