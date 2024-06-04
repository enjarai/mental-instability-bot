use std::time::Duration;

use crate::commands::{modversion::get_mod_version, version::get_yarn_version};

use super::{Context, Error};
use poise::CreateReply;
use serenity::all::Message;
use tokio::time::sleep;

#[poise::command(
    track_edits,
    context_menu_command = "Fix gradle dependencies",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn update_deps(ctx: Context<'_>, msg: Message) -> Result<(), Error> {
    let content = msg.content;

    if let Some(mc_version) = content.lines().find_map(|l| {
        if let Some((id, version)) = l.split_once("=")
            && id == "deps.minecraft"
        {
            Some(version)
        } else {
            None
        }
    }) {
        let eta = content.lines().count() / 4 + 1;
        let reply = ctx
            .send(
                CreateReply::default()
                    .ephemeral(false)
                    .content(if eta > 3 {
                        format!(
                            "Attempting to fix dependencies, eta: {eta} seconds..."
                        )
                    } else {
                        "Attempting to fix dependencies, please wait...".to_string()
                    })
                    .reply(true),
            )
            .await?;

        let mut fixed = vec![];
        for l in content.lines() {
            if !l.contains("`") {
                let new_l = if let Some((id, version)) = l.split_once("=")
                    && let Some((prefix, slug)) = id.split_once(".")
                    && prefix == "deps"
                    && slug != "minecraft"
                {
                    let result = if slug == "yarn" {
                        get_yarn_version(mc_version)
                            .await?
                            .version
                            .ok_or_else(|| "No available Yarn version.".to_string())
                    } else {
                        sleep(Duration::from_millis(200)).await;
                        get_mod_version(slug, Some(mc_version), "fabric").await?
                    };

                    match result {
                        Ok(new_version) => {
                            format!("{id}={new_version}")
                        }
                        Err(err) => {
                            format!("# {err}\n{id}={version}")
                        }
                    }
                } else {
                    l.to_owned()
                };
                fixed.push(new_l);
            }
        }

        let fixed_string = fixed.join("\n");

        reply
            .edit(
                ctx,
                CreateReply::default()
                    .ephemeral(false)
                    .content(format!("```\n{fixed_string}\n```")),
            )
            .await?;
    } else {
        ctx.send(
            CreateReply::default().ephemeral(true).content(
                "Missing minecraft version dependency, create a property called `deps.minecraft` and set it to the desired version.",
            ),
        )
        .await?;
    }

    Ok(())
}
