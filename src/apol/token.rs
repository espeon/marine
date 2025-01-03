use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
#[derive(Debug, Clone)]
pub enum AppleMusicToken {
    Unauthenticated {
        jwt: String,
    },
    Authenticated {
        jwt: String,
        user_token: String,
        store_id: String,
    },
}

#[derive(Deserialize)]
struct Storefronts {
    data: Vec<StorefrontData>,
}

#[derive(Deserialize)]
struct StorefrontData {
    id: String,
}

impl AppleMusicToken {
    pub async fn new() -> Result<Self> {
        let jwt = get_bearer_token().await?;
        Ok(AppleMusicToken::Unauthenticated { jwt })
    }

    pub async fn authenticate(self, user_token: String) -> Result<Self> {
        match self {
            AppleMusicToken::Unauthenticated { jwt } => {
                let store_id = get_storefront(&jwt, &user_token).await?;
                Ok(AppleMusicToken::Authenticated {
                    jwt,
                    user_token,
                    store_id,
                })
            }
            AppleMusicToken::Authenticated { .. } => Err(anyhow!("Token is already authenticated")),
        }
    }

    pub fn get_jwt(&self) -> &str {
        match self {
            AppleMusicToken::Unauthenticated { jwt } => jwt,
            AppleMusicToken::Authenticated { jwt, .. } => jwt,
        }
    }

    pub fn get_user_token(&self) -> Option<&str> {
        match self {
            AppleMusicToken::Unauthenticated { .. } => None,
            AppleMusicToken::Authenticated { user_token, .. } => Some(user_token),
        }
    }

    pub fn get_store_id(&self) -> Option<&str> {
        match self {
            AppleMusicToken::Unauthenticated { .. } => None,
            AppleMusicToken::Authenticated { store_id, .. } => Some(store_id),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, AppleMusicToken::Authenticated { .. })
    }
}

fn create_default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/json;charset=utf-8"),
    );
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert(
        "Origin",
        HeaderValue::from_static("https://music.apple.com"),
    );
    headers.insert(
        "Referer",
        HeaderValue::from_static("https://music.apple.com/"),
    );
    headers.insert(
        "Accept-Encoding",
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36"));
    headers
}

pub async fn get_bearer_token() -> Result<String> {
    let client = reqwest::Client::new();
    let headers = create_default_headers();

    // Get main page
    let main_page_response = client
        .get("https://music.apple.com/us/browse")
        .headers(headers.clone())
        .send()
        .await?;

    if !main_page_response.status().is_success() {
        return Err(anyhow!("Failed to send request to Apple Music"));
    }

    let main_page_code = main_page_response.text().await?;

    // Find JS file
    static JS_SEARCH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"index(.*?)\.js").unwrap());
    let js_file = JS_SEARCH_RE
        .captures(&main_page_code)
        .ok_or_else(|| anyhow!("Failed to find js file"))?
        .get(0)
        .unwrap()
        .as_str();

    // Get JS file content
    let js_file_response = client
        .get(format!("https://music.apple.com/assets/{}", js_file))
        .headers(headers)
        .send()
        .await?;

    if !js_file_response.status().is_success() {
        return Err(anyhow!("Failed to send request to Apple Music"));
    }

    let js_file_code = js_file_response.text().await?;

    const JWT_REGEX: &str = r#""(?P<key>eyJh(.*?))""#;

    // Find JWT
    let jwt_search_re = Regex::new(JWT_REGEX)?;
    let jwt = jwt_search_re
        .captures(&js_file_code)
        .ok_or_else(|| anyhow!("Failed to find jwt"))?
        .name("key")
        .ok_or_else(|| anyhow!("Failed to find jwt key group"))?
        .as_str();

    Ok(jwt.to_string())
}

pub async fn get_storefront(jwt: &str, user_token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let mut headers = create_default_headers();

    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", jwt))?,
    );
    headers.insert("media-user-token", HeaderValue::from_str(user_token)?);

    let storefront_response = client
        .get("https://amp-api.music.apple.com/v1/me/storefront")
        .headers(headers)
        .send()
        .await?;

    if !storefront_response.status().is_success() {
        return Err(anyhow!("Failed to send request to Apple Music"));
    }

    let storefront: Storefronts = storefront_response.json().await?;
    let store_id = storefront
        .data
        .first()
        .ok_or_else(|| anyhow!("No storefront data found"))?
        .id
        .clone();

    Ok(store_id)
}
