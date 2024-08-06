use crate::{constants::MODRINTH_PROJECT_URL, util::create_http};

use super::{Context, Error};
use poise::CreateReply;
use reqwest::StatusCode;
use serde::Deserialize;
use serenity::all::{CreateActionRow, CreateButton, CreateEmbed};
use thousands::Separable;

#[derive(Deserialize)]
pub struct ModrinthProject {
    slug: String,
    title: String,
    description: String,
    icon_url: String,
    color: u32,
    downloads: u32,
}

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn modrinth(
    ctx: Context<'_>,
    #[description = "The slug of the Modrinth project"] slug: String,
) -> Result<(), Error> {
    match get_modrinth_info(&slug).await {
        Ok(Ok(project)) => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title(format!("{}", project.title))
                            .description(format!(
                                "{}\n\n{} Downloads",
                                project.description,
                                project.downloads.separate_with_commas()
                            ))
                            .thumbnail(project.icon_url)
                            .color(project.color),
                    )
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new_link(format!(
                            "{}/{}",
                            MODRINTH_PROJECT_URL, project.slug
                        ))
                        .label("Download"),
                    ])]),
            )
            .await?;
        }
        Ok(Err(err)) => {
            ctx.reply(err).await?;
        }
        Err(err) => {
            ctx.reply(format!("Unknown error: {err}")).await?;
        }
    }

    Ok(())
}

pub async fn get_modrinth_info(slug: &str) -> Result<Result<ModrinthProject, String>, Error> {
    let client = create_http()?;

    match client
        .get(format!(
            "{}/project/{}",
            crate::constants::MODRINTH_API_URL,
            slug,
        ))
        .send()
        .await?
        .error_for_status()
    {
        Ok(modrinth_response) => Ok(Ok(serde_json::from_str(&modrinth_response.text().await?)?)),
        Err(err) => {
            if err.status() == Some(StatusCode::NOT_FOUND) {
                Ok(Err(format!("Not a valid Modrinth project: {slug}")))
            } else {
                return Ok(Err(format!("Unknown status code returned by API: {err}")));
            }
        }
    }
}
