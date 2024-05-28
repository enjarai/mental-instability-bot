use std::fs;

use poise::CreateReply;
use serde::Deserialize;
use serenity::all::InstallationContext;
use serenity::all::InteractionContext;
use serenity::builder::CreateEmbed;

use crate::ConfigData;

use super::Context;
use super::Error;

#[derive(Deserialize)]
struct Tag {
    title: String,
    description: Option<String>,
    thumbnail: Option<String>,
    image: Option<String>,
    color: u32,
}

pub fn load_tag_commands() -> Vec<poise::Command<ConfigData, Error>> {
    let files = fs::read_dir("./tags/").expect("reading tags directory");

    let mut result = vec![];

    for file in files {
        let file = file.expect("locating tag");
        let path = file.path();
        let file_name = file.file_name().into_string().expect("reading filename");

        if file_name.ends_with(".json5") {
            let tag_name = file_name.strip_suffix(".json5").unwrap();
            let tag = json5::from_str::<Tag>(&fs::read_to_string(path).expect("reading tag"))
                .expect("parsing tag");

            result.push(tag_command(String::from(tag_name), tag))
        }
    }

    result
}

fn tag_command(tag_name: String, tag: Tag) -> poise::Command<ConfigData, Error> {
    async fn inner(ctx: Context<'_>) -> Result<(), Error> {
        let data = ctx.command().custom_data.as_ref();
        let tag = data
            .downcast_ref::<Tag>()
            .expect("Tag command broke, what??");

        let mut embed = CreateEmbed::new()
            .title(&tag.title)
            .color(tag.color);
        if let Some(description) = &tag.description {
            embed = embed.description(description);
        }
        if let Some(thumbnail) = &tag.thumbnail {
            embed = embed.thumbnail(thumbnail);
        }
        if let Some(image) = &tag.image {
            embed = embed.image(image);
        }
        let message = CreateReply::default().embed(embed);

        ctx.send(message).await?;
        Ok(())
    }

    poise::Command {
        name: tag_name.clone(),
        description: Some(format!("Displays the {} tag", tag_name)),
        prefix_action: Some(|ctx| {
            Box::pin(async move {
                inner(ctx.into())
                    .await
                    .map_err(|error| poise::FrameworkError::new_command(ctx.into(), error))
            })
        }),
        slash_action: Some(|ctx| {
            Box::pin(async move {
                inner(ctx.into())
                    .await
                    .map_err(|error| poise::FrameworkError::new_command(ctx.into(), error))
            })
        }),
        context_menu_action: None,
        custom_data: Box::new(tag),
        install_context: Some(vec![InstallationContext::Guild, InstallationContext::User]),
        interaction_context: Some(vec![InteractionContext::Guild, InteractionContext::BotDm, InteractionContext::PrivateChannel]),
        ..Default::default()
    }
}
