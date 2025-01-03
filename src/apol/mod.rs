use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use token::AppleMusicToken;
use tokio::sync::Mutex;

pub mod search;
pub mod token;
#[derive(Clone, Debug)]
pub struct TokenData {
    token: AppleMusicToken,
    expiry: SystemTime,
}

pub struct AppleMusicTokenManager {
    current_token: Option<TokenData>,
    token_lifetime: Duration,
}

// Global static instance
static TOKEN_MANAGER: Lazy<Arc<Mutex<AppleMusicTokenManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(AppleMusicTokenManager::new())));

impl AppleMusicTokenManager {
    fn new() -> Self {
        Self {
            current_token: None,
            // Default token lifetime of 1 hour
            token_lifetime: Duration::from_secs(3600),
        }
    }

    fn is_token_valid(&self) -> bool {
        if let Some(token_data) = &self.current_token {
            SystemTime::now() < token_data.expiry
        } else {
            false
        }
    }

    pub async fn get_token(&mut self) -> Result<AppleMusicToken> {
        if self.is_token_valid() {
            return Ok(self.current_token.as_ref().unwrap().token.clone());
        }

        let token = AppleMusicToken::new().await?;

        // Update stored token with new expiry
        self.current_token = Some(TokenData {
            token: token.clone(),
            expiry: SystemTime::now() + self.token_lifetime,
        });

        Ok(token)
    }

    pub fn set_token_lifetime(&mut self, duration: Duration) {
        self.token_lifetime = duration;
    }

    pub fn clear_token(&mut self) {
        self.current_token = None;
    }
}

// Public API functions
pub async fn get_apple_music_token() -> Result<AppleMusicToken> {
    let mut manager = TOKEN_MANAGER.lock().await;
    manager.get_token().await
}

pub async fn set_token_lifetime(duration: Duration) -> Result<()> {
    let mut manager = TOKEN_MANAGER.lock().await;
    manager.set_token_lifetime(duration);
    Ok(())
}

pub async fn clear_token() -> Result<()> {
    let mut manager = TOKEN_MANAGER.lock().await;
    manager.clear_token();
    Ok(())
}
