use std::path::Path;

use crate::get_config;

use super::{Context, Error};
use serenity::all::{Attachment, Message};
use serenity::{
    all::ChannelId,
    builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage},
};

fn is_image(filename: &str) -> bool {
    Path::new(filename).extension().map_or(false, |ext| {
        ext.eq_ignore_ascii_case("png")
            || ext.eq_ignore_ascii_case("jpg")
            || ext.eq_ignore_ascii_case("jpeg")
            || ext.eq_ignore_ascii_case("gif")
    })
}

async fn quote_internal(
    ctx: Context<'_>,
    author: &String,
    quote: &String,
    icon_url: Option<&String>,
    attachments: Option<&Vec<Attachment>>,
) -> Result<(), Error> {
    match get_config!(ctx.serenity_context()).quotes_channel {
        Some(id) => {
            let channel = ChannelId::new(id);

            if ctx.guild_id().is_none() {
                ctx.reply("Nuh uh :brombeere:").await?;

                return Ok(());
            }

            let mut embed_author = CreateEmbedAuthor::new(author);

            if let Some(icon) = icon_url {
                embed_author = embed_author.icon_url(icon);
            }

            let mut embed = CreateEmbed::new().description(quote).author(embed_author);

            if let Some(attachments) = attachments
                && !attachments.is_empty()
                && is_image(&attachments[0].filename)
            {
                embed = embed.image(&attachments[0].url);
            }

            let builder = CreateMessage::new().embed(embed);

            match channel.send_message(ctx.http(), builder).await {
                Ok(_) => {
                    ctx.reply(format!("Quoted: '{quote}' - {author}")).await?;
                }
                Err(e) => {
                    ctx.reply(format!("Failed to quote, {e}")).await?;
                }
            };
        }
        None => {
            ctx.reply("No quotes channel specified").await?;
        }
    }
    Ok(())
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn quote(
    ctx: Context<'_>,
    #[description = "The text to quote"] quote: String,
    #[description = "The author of said text"] author: String,
) -> Result<(), Error> {
    quote_internal(ctx, &author, &quote, None, None).await
}

#[poise::command(track_edits, context_menu_command = "Quote this message")]
pub async fn context_quote(ctx: Context<'_>, msg: Message) -> Result<(), Error> {
    quote_internal(
        ctx,
        &msg.author.name,
        &msg.content,
        Some(
            &msg.author
                .avatar_url()
                .unwrap_or(msg.author.default_avatar_url()),
        ),
        Some(&msg.attachments),
    )
    .await
}
