use poise::CreateReply;

use super::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn gay(
    ctx: Context<'_>,
) -> Result<(), Error> {
    match &ctx.serenity_context()
            .data
            .read()
            .await
            .get::<crate::ConfigData>()
            .expect("No config?")
            .gay_roles {
        Some(gays) => {
            match ctx.guild_id() {
                Some(id) => {
                    let id: u64 = id.into();
                    match gays.get(&id.to_string()) {
                        Some(role) => {
                            let has_role = ctx.author().has_role(ctx, id, *role).await?;

                            if has_role {
                                ctx.author_member().await.unwrap().remove_role(ctx, *role).await?;
                            } else {
                                ctx.author_member().await.unwrap().add_role(ctx, *role).await?;
                            }

                            ctx.send(CreateReply::default()
                                .ephemeral(true)
                                .content(if has_role {
                                    "Removed gay access"
                                } else {
                                    "Granted gay access"
                                })
                            ).await?;
                        },
                        None => {
                            ctx.reply("No gay role configured").await?;
                        },
                    }
                },
                None => {
                    ctx.reply("Not in a guild").await?;
                },
            }
        },
        None => {
            ctx.reply("No gay roles specified").await?;
        },
    }

    Ok(())
}