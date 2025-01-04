use ::serenity::all::GatewayIntents;
use ::serenity::prelude::TypeMapKey;
use dotenvy::dotenv;
use err::AppError;
use helpers::HttpKey;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use std::env;
use std::sync::Arc;
use tracing::{error, info, warn};

mod apol;
mod err;
mod helpers;
mod odesli;
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

async fn on_error(error: poise::FrameworkError<'_, Arc<Data>, AppError>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            warn!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
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
                voice::queue::skip(),
                voice::queue::now_playing(),
                voice::queue::queue(),
            ],
            pre_command: |ctx| {
                Box::pin(async move {
                    info!("Executing command {}...", ctx.command().qualified_name);
                })
            },
            on_error: |error| Box::pin(on_error(error)),

            post_command: |ctx| {
                Box::pin(async move {
                    info!("Executed command {}!", ctx.command().qualified_name);
                })
            },
            event_handler: |_ctx, event, _framework, _data| {
                Box::pin(async move {
                    info!(
                        "Got an event in event handler: {:?}",
                        event.snake_case_name()
                    );
                    Ok(())
                })
            },

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
