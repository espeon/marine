use std::sync::Arc;

use serenity::all::{ChannelId, GuildId};
use songbird::Songbird;

use crate::{err::AppError, Context};

pub mod metadata;
pub mod pause;
pub mod play;
pub mod queue;

pub async fn guild_info(ctx: Context<'_>) -> Result<(GuildId, ChannelId), AppError> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow::anyhow!("Not in a guild"))?;

    let channel_id = ctx.channel_id();

    Ok((guild_id, channel_id))
}

pub async fn init_call(
    manager: &Arc<Songbird>,
    guild_id: serenity::model::id::GuildId,
    channel_id: serenity::model::id::ChannelId,
) -> std::result::Result<
    std::sync::Arc<serenity::prelude::Mutex<songbird::Call>>,
    songbird::error::JoinError,
> {
    if manager.get(guild_id).is_none() {
        return manager.join(guild_id, channel_id).await;
    }

    Err(songbird::error::JoinError::NoCall)
}

pub async fn get_or_join_call(
    manager: &Arc<Songbird>,
    guild_id: serenity::model::id::GuildId,
    channel_id: serenity::model::id::ChannelId,
) -> std::result::Result<
    std::sync::Arc<serenity::prelude::Mutex<songbird::Call>>,
    songbird::error::JoinError,
> {
    if let Ok(call) = init_call(&manager, guild_id, channel_id).await {
        Ok(call)
    } else {
        manager.join(guild_id, channel_id).await
    }
}
