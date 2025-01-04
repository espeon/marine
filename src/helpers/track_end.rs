use std::sync::Arc;

use serenity::{
    all::{ChannelId, CreateMessage, Http, MessageBuilder},
    async_trait,
};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};
use tracing::warn;

use crate::{helpers::trim_artist_from_title, voice::metadata::Metadata};

pub struct TrackEndNotifier {
    pub chan_id: ChannelId,
    pub http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track(track_list) = ctx else {
            return None;
        };

        let track = track_list.get(1)?;

        let typemap = track.1.typemap().read().await;
        let metadata = typemap.get::<Metadata>()?;

        let mut msg = MessageBuilder::new();

        if let (Some(title), artist) = (&metadata.title, &metadata.artist) {
            msg.push_bold_safe(trim_artist_from_title(
                title,
                artist.as_deref().unwrap_or("MY CLOCK"),
            ));

            if let Some(artist) = artist {
                msg.push(" - ").push_bold_safe(artist);
            }

            msg.push(" now playing");

            if let Err(e) = self
                .chan_id
                .send_message(&self.http, CreateMessage::new().content(msg.build()))
                .await
            {
                warn!("Failed to send message: {}", e);
            }
        }

        None
    }
}
