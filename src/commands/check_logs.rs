use crate::log_upload::check_for_logs;

use super::{Context, Error};
use serenity::all::Message;

#[poise::command(track_edits, context_menu_command = "Check for logs")]
pub async fn check_logs(ctx: Context<'_>, msg: Message) -> Result<(), Error> {
    match check_for_logs(&ctx.serenity_context(), &msg, true).await {
        Ok(count) => {
            if count > 0 {
                ctx.reply(format!("Found {} logs, uploaded.", count)).await?;
            } else {
                ctx.reply("No logs detected.").await?;
            }
        },
        Err(err) => {
            ctx.reply(format!("Error, cannot upload logs: {}", err)).await?;
        },
    };
    Ok(())
}