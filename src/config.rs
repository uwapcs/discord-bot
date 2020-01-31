use serenity;
use std::fs;
use serde::Deserialize;
use toml;

lazy_static! {
    static ref CONFIG_FILE: String = fs::read_to_string("config.toml").unwrap();
}

lazy_static! {
    pub static ref CONFIG: UccbotConfig = toml::from_str(&CONFIG_FILE).unwrap();
}

#[derive(Deserialize)]
pub struct UccbotConfig {
    pub server_id: u64,
    // #general
    pub main_channel: serenity::model::id::ChannelId,
    // #the-corner
    pub welcome_channel: serenity::model::id::ChannelId,
    // #general
    pub announcement_channel: serenity::model::id::ChannelId,
    pub bot_id: u64,
    pub vote_pool_size: i8,
    pub vote_role: u64,
    pub tiebreaker_role: u64,
    pub unregistered_member_role: u64,
    pub registered_member_role: u64,
    pub command_prefix: &'static str,
    pub for_vote: &'static str,
    pub against_vote: &'static str,
    pub abstain_vote: &'static str,
    pub approve_react: &'static str,
    pub disapprove_react: &'static str,
    pub unsure_react: &'static str,
}

impl UccbotConfig {
    pub fn allowed_reacts(&self) -> Vec<String> {
        vec!(self.for_vote.to_string(),
        self.against_vote.to_string(),
        self.abstain_vote.to_string(),
        self.approve_react.to_string(),
        self.disapprove_react.to_string(),
        self.unsure_react.to_string())
    }
}