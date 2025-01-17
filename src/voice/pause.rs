use crate::{voice::metadata::Metadata, AppError, Context};

use super::get_or_join_call;

#[poise::command(
    category = "Music",
    slash_command,
    prefix_command,
    aliases("s"),
    guild_only
)]
pub async fn pause(ctx: Context<'_>) -> Result<(), AppError> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (guild_id, channel_id) = super::guild_info(ctx).await?;

    if let Ok(handler_lock) = get_or_join_call(&manager, ctx, guild_id, channel_id).await {
        let handler = handler_lock.lock().await;

        let current = handler.queue().current();

        if let Err(result) = handler.queue().pause() {
            ctx.say(format!("Failed to pause: {:?}", result)).await?;
        }

        if let Some(current) = current {
            if let Some(metadata) = current.typemap().read().await.get::<Metadata>() {
                ctx.say(format!("Paused - current track is: {:?}", metadata.title))
                    .await?;
            } else {
                ctx.say("Paused this track.").await?;
            }
        } else {
            ctx.say("Paused - no track playing").await?;
        }
    } else {
        ctx.say("Not in a voice channel to pause").await?;
    }

    Ok(())
}
