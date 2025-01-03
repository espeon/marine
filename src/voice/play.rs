use songbird::input::{Compose, YoutubeDl};

use crate::{helpers::get_http_client, AppError, Context};

use super::{get_or_join_call, metadata::Metadata};

#[poise::command(
    category = "Music",
    slash_command,
    prefix_command,
    aliases("s"),
    guild_only
)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "The YouTube URL of the song to play"] song: Option<String>,
) -> Result<(), AppError> {
    if let Some(song) = song {
        play_inner(ctx, song).await
    } else {
        ctx.say("No song provided").await?;
        Ok(())
    }
}

pub async fn resume(ctx: Context<'_>) -> Result<(), AppError> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (guild_id, channel_id) = super::guild_info(ctx).await?;

    if let Ok(handler_lock) = get_or_join_call(&manager, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;

        let current = handler.queue().current();

        if let Err(result) = handler.queue().resume() {
            ctx.say(format!("Failed to unpause: {:?}", result)).await?;
        }

        if let Some(current) = current {
            ctx.say(format!(
                "Unpaused - {:?}",
                current
                    .typemap()
                    .read()
                    .await
                    .get::<Metadata>()
                    .unwrap()
                    .title
            ))
            .await?;
        } else {
            ctx.say("Resumed!").await?;
        }
    } else {
        ctx.say("Not in a voice channel to resume").await?;
    }

    Ok(())
}

pub async fn play_inner(ctx: Context<'_>, song: String) -> Result<(), AppError> {
    // check to make sure the url is valid
    if !song.starts_with("https://") {
        return Err(anyhow::anyhow!("Invalid URL").into());
    }

    // make sure songbird has been initialized
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let channel_id = ctx.channel_id();

    if let Ok(handler_lock) = get_or_join_call(&manager, guild_id, channel_id).await {
        let mut handler = handler_lock.lock().await;

        let http = get_http_client(ctx.serenity_context()).await;

        // todo: variably create source based on url/track source
        let mut src = YoutubeDl::new(http, song);

        let metadata = src.aux_metadata().await;

        let h = handler.enqueue_input(src.into()).await;

        if let Ok(metadata) = metadata {
            ctx.say(format!("Playing - {:?}", metadata.title)).await?;
            h.typemap().write().await.insert::<Metadata>(metadata);
        } else {
            ctx.say("Failed to get metadata, but playing anyways")
                .await?;
        }
    } else {
        ctx.say("Not in a voice channel to play in").await?;
    }

    Ok(())
}
