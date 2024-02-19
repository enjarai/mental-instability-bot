use anyhow::Result;
use serenity::{all::GuildChannel, client::Context, http::CacheHttp};

use crate::get_config;

pub async fn add_owner_to_thread(ctx: &Context, thread: &GuildChannel) -> Result<()> {
    let config = {
        ctx.data
            .read()
            .await
            .get::<crate::Data>()
            .expect("No config?")
    };
    let test = &config.thread_joiners;

    Ok(())
}