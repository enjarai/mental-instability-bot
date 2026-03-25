use crate::constants::TRICKSTER_STUDIO_URL;

use super::{Context, Error};
use urlencoding::encode;

/// Generate a link to view a Trickster spell
#[poise::command(slash_command, prefix_command)]
pub async fn view_spell(
    ctx: Context<'_>,
    #[description = "The Spell Export String"] spell: String,
) -> Result<(), Error> {
    let url_encoded_spell = encode(&spell);
    let url = format!("[Click to view spell in browser]({}/?spell={})", TRICKSTER_STUDIO_URL, url_encoded_spell);
    ctx.reply(url).await?;
    Ok(())
}
