use ::serenity::all::{EventHandler, GatewayIntents, Message};
use ::serenity::prelude::TypeMapKey;
use dotenvy::dotenv;
use err::AppError;
use helpers::HttpKey;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use std::env;
use std::sync::Arc;
use tracing::info;

mod apol;
mod err;
mod helpers;
mod voice;

struct Data {}

impl TypeMapKey for Data {
    type Value = Arc<Data>;
}
type Context<'a> = poise::Context<'a, Arc<Data>, AppError>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), AppError> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

// Event handler
struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: serenity::prelude::Context, _msg: Message) {
        // are we mentioned?
        // get autorespond channels list from env
    }

    async fn ready(&self, ctx: serenity::prelude::Context, _ready: serenity::Ready) {
        let user = ctx.cache.current_user();
        println!("{}: We are up and running.", user.name)
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let discord_token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let user_data = Arc::new(Data {});

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let ud_clone = user_data.clone();
    let framework: poise::Framework<Arc<Data>, AppError> = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                age(),
                voice::play::play(),
                voice::pause::pause(),
                voice::queue::now_playing(),
            ],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                //poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                // for all guilds we are in
                for guild in ctx.cache.guilds() {
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, guild)
                        .await?;
                }
                info!(
                    "{} [{}] connected successfully!",
                    ready.user.name, ready.user.id
                );
                Ok(ud_clone)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        //.event_handler(Handler)
        .register_songbird()
        .type_map_insert::<HttpKey>(reqwest::Client::new())
        .await
        .expect("create client failed");

    {
        let mut data = client.data.write().await;
        data.insert::<Data>(user_data);
        //
    }
    client.start().await.unwrap();
}
