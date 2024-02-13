use super::{Context, Error};
use serenity::all::Message;
use serenity::{
    all::ChannelId,
    builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage},
};

async fn quote_internal(
    ctx: Context<'_>,
    author: &String,
    quote: &String,
    icon_url: Option<String>,
) -> Result<(), Error> {
    match ctx.data().config.quotes_channel {
        Some(id) => {
            let mut embed_author = CreateEmbedAuthor::new(author);

            if let Some(icon) = icon_url {
                embed_author = embed_author.icon_url(icon);
            }

            let embed = CreateEmbed::new().description(quote).author(embed_author);
            let builder = CreateMessage::new().embed(embed);
            match ChannelId::new(id).send_message(ctx.http(), builder).await {
                Ok(_) => {
                    ctx.reply(format!("Quoted: '{quote}'")).await?;
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
    quote_internal(ctx, &author, &quote, Option::None).await
}

#[poise::command(track_edits, context_menu_command = "Quote this message")]
pub async fn context_quote(ctx: Context<'_>, msg: Message) -> Result<(), Error> {
    quote_internal(
        ctx,
        &msg.author.name,
        &msg.content,
        Option::Some(
            msg.author
                .avatar_url()
                .unwrap_or(msg.author.default_avatar_url()),
        ),
    )
    .await
}
