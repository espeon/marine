use std::cmp::min;

use poise::CreateReply;

//nowplaying
use crate::{
    helpers::d2hms,
    voice::{metadata::Metadata, play::build_play_embed},
    AppError, Context,
};

use super::get_or_join_call;

#[poise::command(
    category = "Music",
    slash_command,
    prefix_command,
    aliases("s"),
    guild_only
)]
pub async fn skip(ctx: Context<'_>) -> Result<(), AppError> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (guild_id, channel_id) = super::guild_info(ctx).await?;

    if let Ok(handler_lock) = get_or_join_call(&manager, ctx, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;
        if let Err(result) = handler.queue().skip() {
            ctx.say(format!("Failed to skip: {:?}", result)).await?;
            return Ok(());
        }
        let current = handler.queue().current();
        if let Some(current) = current {
            if let Some(metadata) = current.typemap().read().await.get::<Metadata>() {
                let msg = format!("Skipped, now playing: {:?}", metadata.title);
                let embed = build_play_embed(metadata, false, None).await;
                ctx.send(CreateReply::default().embed(embed).content(msg))
                    .await?;
            } else {
                ctx.say("Skipped this track.").await?;
            }
        } else {
            ctx.say("Couldn't get metadata for this track").await?;
        }
    } else {
        ctx.say("Not in a voice channel!").await?;
    }

    Ok(())
}

#[poise::command(
    category = "Music",
    slash_command,
    prefix_command,
    aliases("np"),
    guild_only
)]
pub async fn now_playing(ctx: Context<'_>) -> Result<(), AppError> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (guild_id, channel_id) = super::guild_info(ctx).await?;

    if let Ok(handler_lock) = get_or_join_call(&manager, ctx, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;

        let current = handler.queue().current();

        if let Some(current) = current {
            if let Some(metadata) = current.typemap().read().await.get::<Metadata>() {
                let position = current.get_info().await.map(|i| i.position).ok();
                let embed = build_play_embed(metadata, true, position).await;
                ctx.send(CreateReply::default().embed(embed)).await?;
            } else if let Ok(info) = current.get_info().await {
                ctx.say(format!(
                    "Current position is: {} / {}",
                    d2hms(info.play_time),
                    d2hms(info.position)
                ))
                .await?;
            } else {
                ctx.say("No metadata available!").await?;
            }
        } else {
            ctx.say("No track playing!").await?;
        }
    } else {
        ctx.say("Not in a voice channel!").await?;
    }

    Ok(())
}

#[poise::command(
    category = "Music",
    slash_command,
    prefix_command,
    aliases("q"),
    guild_only
)]
pub async fn queue(ctx: Context<'_>) -> Result<(), AppError> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (guild_id, channel_id) = super::guild_info(ctx).await?;

    if let Ok(handler_lock) = get_or_join_call(&manager, ctx, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;

        let queue = handler.queue().current_queue();

        let mut msg = String::new();
        let from = 0;
        let to = min(from + 10, queue.len());
        msg.push_str(&format!(
            "Queue, page {} of {}\n",
            (from + 9) / 10 + 1,
            (queue.len() + 9) / 10
        ));
        for (i, track) in queue[from..to].iter().enumerate() {
            if let Some(metadata) = track.typemap().read().await.get::<Metadata>() {
                let title = metadata.title.clone().unwrap_or("This track".to_string());
                msg.push_str(&format!("{}. {}\n", i + 1, title));
            }
        }

        if msg.is_empty() {
            ctx.say("Queue is empty").await?;
        } else {
            ctx.say(msg).await?;
        }
    } else {
        ctx.say("Not in a voice channel!").await?;
    }

    Ok(())
}

#[poise::command(
    category = "Music",
    slash_command,
    prefix_command,
    aliases("rm"),
    guild_only
)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The index of the track to remove"] index: usize,
) -> Result<(), AppError> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (guild_id, channel_id) = super::guild_info(ctx).await?;

    if let Ok(handler_lock) = get_or_join_call(&manager, ctx, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;

        let queue = handler.queue().modify_queue(|queue| queue.remove(index));

        if let Some(current) = queue {
            if let Some(metadata) = current.typemap().read().await.get::<Metadata>() {
                ctx.say(format!(
                    "Successfully removed {} at index {}",
                    metadata.title.clone().unwrap_or("the track".to_owned()),
                    index
                ))
                .await?;
            }
        } else {
            ctx.say(format!("Failed to remove the track at index {}", index))
                .await?;
        }
    } else {
        ctx.say("Not in a voice channel!").await?;
    }

    Ok(())
}
