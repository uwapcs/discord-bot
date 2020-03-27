use base64;
use chrono::{prelude::Utc, DateTime};
use openssl::symm::{decrypt, encrypt, Cipher};
use rand::Rng;
use serenity::model::user::User;
use std::str;

lazy_static! {
    static ref KEY: [u8; 32] = rand::thread_rng().gen::<[u8; 32]>();
    static ref CIPHER: Cipher = Cipher::aes_256_cbc();
}

fn text_encrypt(plaintext: &str) -> String {
    let iv: &[u8; 16] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let encrypted_vec =
        encrypt(*CIPHER, &*KEY, Some(iv), plaintext.as_bytes()).expect("encryption failed");
    base64::encode(encrypted_vec.as_slice())
}
fn text_decrypt(ciphertext: &str) -> Option<String> {
    let iv: &[u8; 16] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    guard!(let Ok(cipher_vec) = base64::decode(ciphertext) else {
        warn!("Unable to decode base64 text");
        return None
    });
    guard!(let Ok(decrypted_vec) = decrypt(*CIPHER, &*KEY, Some(iv), &cipher_vec) else {
        warn!("Text decryption failed");
        return None
    });
    guard!(let Ok(decrypted_token) = str::from_utf8(decrypted_vec.as_slice()) else {
        warn!("Invalid utf8 in text");
        return None
    });
    Some(decrypted_token.to_owned())
}

pub fn generate_token(discord_user: &User, username: &str) -> String {
    // if username doesn't exist : throw error
    let timestamp = Utc::now().to_rfc3339();
    let payload = format!(
        "{},{},{}",
        timestamp,
        discord_user.id.0.to_string(),
        username
    );
    info!("Token generated for {}: {}", discord_user.name, &payload);
    text_encrypt(&payload)
}

#[derive(Debug)]
pub enum TokenError {
    DiscordIdMismatch,
    TokenExpired,
    TokenInvalid,
}
impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub static TOKEN_LIFETIME: i64 = 300; // 5 minutes

pub fn parse_token(discord_user: &User, encrypted_token: &str) -> Result<String, TokenError> {
    guard!(let Some(token) = text_decrypt(encrypted_token) else {
        return Err(TokenError::TokenInvalid)
    });
    let token_components: Vec<_> = token.splitn(3, ',').collect();
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
        return Err(TokenError::DiscordIdMismatch)
    }
    let time_delta_seconds = Utc::now().timestamp() - token_timestamp.timestamp();
    if time_delta_seconds > TOKEN_LIFETIME {
        warn!(
            "... attempt failed : token expired ({} seconds old)",
            time_delta_seconds
        );
        return Err(TokenError::TokenExpired)
    }
    info!(
        "... verification successful (token {} seconds old)",
        time_delta_seconds
    );
    Ok(token_username.to_owned())
}
