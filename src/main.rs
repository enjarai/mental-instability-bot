#![feature(let_chains)]

mod commands;
mod config;
mod constants;
mod log_upload;
mod macros;
// mod mapping;
mod thread_adder;

use std::fs;

use config::Config;
use log_upload::check_for_logs;
use poise::FrameworkOptions;
use serenity::all::GuildChannel;
use serenity::all::Message;
use serenity::all::Ready;
use serenity::async_trait;
use serenity::prelude::*;
use thread_adder::add_owner_to_thread;

pub struct Data;

impl TypeMapKey for Data {
    type Value = Config;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, event: Ready) {
        println!("Bot ready! Logged in as {}", event.user.name);
    }

    async fn message(&self, ctx: Context, message: Message) {
        let _ = check_for_logs(&ctx, &message).await;
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        let _ = add_owner_to_thread(&ctx, &thread).await;
    }
}

#[tokio::main]
async fn main() {
    let mut commands = vec![
        commands::general::register(),
        commands::quote::quote(),
        commands::quote::context_quote(),
        commands::version::version(),
    ];
    commands.append(&mut commands::tags::load_tag_commands());

    let poise_options = FrameworkOptions {
        commands,
        on_error: |err| {
            Box::pin(async move {
                println!("{}", err);
            })
        },
        ..Default::default()
    };

    let config: Config =
        toml::from_str(&fs::read_to_string("config.toml").expect("reading config"))
            .expect("parsing config");

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Registering commands");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .options(poise_options)
        .build();

    // Login with a bot token from the environment
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&config.token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    client.data.write().await.insert::<Data>(config);

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {why:?}");
    }
}
