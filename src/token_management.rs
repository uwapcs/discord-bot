use chrono::{
    prelude::{SecondsFormat, Utc},
    DateTime,
};
use rand::Rng;
use serenity::model::user::User;
use std::str;

lazy_static! {
    static ref key: [u8; 32] = rand::thread_rng().gen::<[u8; 32]>();
}

fn encrypt(plaintext: &str) -> &str {
    return plaintext;
}
fn decrypt(ciphertext: &str) -> &str {
    return ciphertext;
}

pub fn generate_token<'a>(discord_user: &User, username: &str) -> String {
    // if username doesn't exist : throw error
    let timestamp = Utc::now().to_rfc3339();
    let payload = format!(
        "{},{},{}",
        timestamp,
        discord_user.id.0.to_string(),
        username
    );
    info!("Token generated for {}: {}", discord_user.name, &payload);
    encrypt(&payload).to_string()
}

#[derive(Debug)]
pub enum TokenError {
    DiscordIdMismatch,
    TokenExpired,
}
impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn parse_token(discord_user: &User, token: &str) -> Result<String, TokenError> {
    let token_components: Vec<_> = decrypt(token).splitn(3, ',').collect();
    info!(
        "Verification attempt from '{}'(uid: {}) for account '{}' with token from {}",
        discord_user.name, token_components[1], token_components[2], token_components[0]
    );
    let token_timestamp =
        DateTime::parse_from_rfc3339(token_components[0]).expect("Invalid date format");
    let token_discord_user = token_components[1];
    let token_username = token_components[2];
    if token_discord_user != discord_user.id.0.to_string() {
        warn!("... attempt failed : DiscordID mismatch");
        return Err(TokenError::DiscordIdMismatch);
    }
    let time_delta_seconds = Utc::now().timestamp() - token_timestamp.timestamp();
    if time_delta_seconds > 5 * 60 {
        warn!(
            "... attempt failed : token expired ({} seconds old)",
            time_delta_seconds
        );
        return Err(TokenError::TokenExpired);
    }
    info!(
        "... verification successful (token {} seconds old)",
        time_delta_seconds
    );
    return Ok(token_username.to_owned());
}
