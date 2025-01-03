//nowplaying
use crate::{voice::metadata::Metadata, AppError, Context};

use super::get_or_join_call;

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

    if let Ok(handler_lock) = get_or_join_call(&manager, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;

        let current = handler.queue().current();

        if let Some(current) = current {
            ctx.say(format!(
                "Current track is: {:?}",
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
            ctx.say("Couldn't get metadata for this track").await?;
        }
    } else {
        ctx.say("Not in a voice channel!").await?;
    }

    Ok(())
}
