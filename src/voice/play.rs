use std::time::Duration;

use poise::CreateReply;
use serenity::all::CreateEmbed;
use songbird::input::{AuxMetadata, Compose, YoutubeDl};
use tracing::info;

use crate::{
    helpers::{d2hms, get_http_client, trim_artist_from_title},
    AppError, Context,
};

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
        resume(ctx).await
    }
}

pub async fn resume(ctx: Context<'_>) -> Result<(), AppError> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (guild_id, channel_id) = super::guild_info(ctx).await?;

    if let Ok(handler_lock) = get_or_join_call(&manager, ctx, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;

        let current = handler.queue().current();

        if let Err(result) = handler.queue().resume() {
            ctx.say(format!("Failed to unpause: {:?}", result)).await?;
        }

        if let Some(current) = current {
            if let Some(metadata) = current.typemap().read().await.get::<Metadata>() {
                ctx.say(format!(
                    "Unpaused - {}",
                    trim_artist_from_title(
                        &metadata.title.clone().unwrap_or("None".to_string()),
                        &metadata.artist.clone().unwrap_or("MY CLOCK".to_string())
                    )
                ))
                .await?;
            }
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

    if let Ok(handler_lock) = get_or_join_call(&manager, ctx, guild_id, channel_id).await {
        // give us more time to load the track!
        ctx.defer().await?;
        let mut handler = handler_lock.lock().await;

        let http = get_http_client(ctx.serenity_context()).await;

        // todo: variably create source based on url/track source
        let mut src = YoutubeDl::new(http, song);

        let metadata = src.aux_metadata().await;

        if let Ok(metadata) = metadata {
            info!("Got metadata: {:?}", metadata);

            let embed = build_play_embed(&metadata, false, None).await;

            let mut title = metadata.title.clone().unwrap_or("This track".to_string());

            if title != "This track" {
                title = trim_artist_from_title(
                    &title,
                    &metadata.artist.clone().unwrap_or("MY CLOCK".to_string()),
                );
            }

            // build reply message
            let queue = handler.queue();
            let content = match queue.len() {
                0 => format!("{title} is now playing."),
                1 => format!("{title} is up next."),
                2 => format!("{title} will play after this next track."),
                _ => format!(
                    "{title} will play after the next {} tracks.",
                    queue.len() - 1
                ),
            };

            let reply = CreateReply::default().content(content).embed(embed);

            ctx.send(reply).await?;
            let h = handler.enqueue_input(src.into()).await;
            h.typemap()
                .write()
                .await
                .insert::<Metadata>(metadata.clone());
        } else {
            info!("Failed to get metadata or no metadata available");
            ctx.say("Failed to get metadata, but playing anyways")
                .await?;
            handler.enqueue_input(src.into()).await;
        }
    } else {
        ctx.say("Not in a voice channel to play in").await?;
    }

    Ok(())
}

pub async fn build_play_embed(
    metadata: &AuxMetadata,
    title: bool,
    progress: Option<Duration>,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let mut desc = String::new();
    if title {
        if let Some(title) = metadata.title.clone() {
            embed = embed.title(title);
        }
    }
    if let Some(thumbnail) = metadata.thumbnail.clone() {
        embed = embed.image(thumbnail);
    }
    if let Some(duration) = metadata.duration {
        if let Some(progress) = progress {
            desc.push_str(&format!("{} / ", d2hms(progress)));
        }
        desc.push_str(&format!("{}\n", d2hms(duration)));
    }
    if let Some(source_url) = metadata.source_url.clone() {
        embed = embed.url(source_url);
    }
    if let Some(artist) = metadata.artist.clone() {
        desc.push_str(&format!("Artist: {}\n", artist));
    }
    if let Some(album) = metadata.album.clone() {
        desc.push_str(&format!("Album: {}\n", album));
    }
    if !desc.is_empty() {
        embed = embed.description(desc);
    }
    embed
}
