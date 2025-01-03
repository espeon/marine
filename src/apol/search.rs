use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, ORIGIN, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::apol::get_apple_music_token;

// Modified search function
pub async fn search_track(query: String) -> Result<Option<AppleMusicSong>> {
    println!("searching for {}", query);

    let tk = get_apple_music_token().await?;

    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, HeaderValue::from_static("https://music.apple.com"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36"
        ),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", tk.get_jwt()))?,
    );

    let client = reqwest::Client::new();
    let url = format!(
        "https://amp-api.music.apple.com/v1/catalog/{}/search?term={}&limit=1&types=songs",
        "us",
        urlencoding::encode(&query)
    );

    let response = client.get(&url).headers(headers).send().await?;

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        let songs = json
            .get("results")
            .and_then(|v| v.get("songs"))
            .and_then(|v| serde_json::from_value(v.clone()).ok());
        Ok(songs)
    } else {
        let status = response.status();
        let text = response.text().await?;
        Err(anyhow::anyhow!("{}: {}", status, text))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleMusicSong {
    pub data: Option<Vec<AppleMusicSongDatum>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleMusicSongDatum {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub href: Option<String>,
    pub attributes: Option<Attributes>,
    pub relationships: Option<Relationships>,
    pub meta: Option<Meta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attributes {
    pub album_name: Option<String>,
    pub has_time_synced_lyrics: Option<bool>,
    pub genre_names: Option<Vec<String>>,
    pub track_number: Option<i32>,
    pub duration_in_millis: Option<i64>,
    pub is_vocal_attenuation_allowed: Option<bool>,
    pub is_mastered_for_itunes: Option<bool>,
    pub isrc: Option<String>,
    pub artwork: Option<Artwork>,
    pub composer_name: Option<String>,
    pub audio_locale: Option<String>,
    pub url: Option<String>,
    pub play_params: Option<PlayParams>,
    pub disc_number: Option<i32>,
    pub has_lyrics: Option<bool>,
    pub is_apple_digital_master: Option<bool>,
    pub audio_traits: Option<Vec<String>>,
    pub name: Option<String>,
    pub previews: Option<Vec<Preview>>,
    pub artist_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artwork {
    pub width: Option<i32>,
    pub url: Option<String>,
    pub height: Option<i32>,
    pub text_color3: Option<String>,
    pub text_color2: Option<String>,
    pub text_color4: Option<String>,
    pub text_color1: Option<String>,
    pub bg_color: Option<String>,
    pub has_p3: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayParams {
    pub id: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preview {
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub content_version: Option<ContentVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentVersion {
    pub mz_indexer: Option<i32>,
    pub rtci: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationships {
    pub albums: Option<Albums>,
    pub artists: Option<Albums>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Albums {
    pub href: Option<String>,
    pub data: Option<Vec<AlbumsDatum>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumsDatum {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub href: Option<String>,
}
