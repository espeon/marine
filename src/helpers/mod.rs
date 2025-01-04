use std::time::Duration;

use reqwest::Client as HttpClient;
use serenity::{all::Context, prelude::TypeMapKey};

pub mod track_end;

pub struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

pub async fn get_http_client(ctx: &Context) -> HttpClient {
    let data = ctx.data.read().await;
    data.get::<HttpKey>()
        .cloned()
        .expect("Guaranteed to exist in the typemap.")
}

pub fn s2hms(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
pub fn d2hms(duration: Duration) -> String {
    s2hms(duration.as_secs())
}

/// Trims the artist from the title, if it's present.
/// If the title is "Artist - Title", it will return "Title".
/// If the title is "Title - Artist", it will return "Title".
pub fn trim_artist_from_title(title: &str, artist: &str) -> String {
    // find 'artist - '
    if let Some(pos) = title.find((artist.to_owned() + " - ").as_str()) {
        return title[pos + artist.len() + 3..].to_owned();
    }
    if let Some(pos) = title.find((" - ".to_string() + artist).as_str()) {
        return title[pos + 3..].to_owned();
    }

    title.to_owned()
}
