use serde::Deserialize;
use serde_yaml;
use serenity::model::id;
use indexmap::IndexMap;
use std::fs;

lazy_static! {
    static ref CONFIG_FILE: String = fs::read_to_string("config.yml").unwrap();
    pub static ref CONFIG: UccbotConfig = serde_yaml::from_str(&CONFIG_FILE).unwrap();
}

#[derive(Debug, Deserialize)]
pub struct UccbotConfig {
    pub server_id: u64,
    pub main_channel: id::ChannelId,
    pub welcome_channel: id::ChannelId,
    pub announcement_channel: id::ChannelId,
    pub bot_id: u64,
    pub vote_pool_size: i8,
    pub vote_role: u64,
    pub tiebreaker_role: u64,
    pub unregistered_member_role: u64,
    pub registered_member_role: u64,
    pub command_prefix: String,
    pub for_vote: String,
    pub against_vote: String,
    pub abstain_vote: String,
    pub approve_react: String,
    pub disapprove_react: String,
    pub unsure_react: String,
    pub react_role_messages: Vec<ReactionMapping>,
}

impl UccbotConfig {
    pub fn allowed_reacts(&self) -> Vec<String> {
        vec![
            self.for_vote.to_string(),
            self.against_vote.to_string(),
            self.abstain_vote.to_string(),
            self.approve_react.to_string(),
            self.disapprove_react.to_string(),
            self.unsure_react.to_string(),
        ]
    }
}

pub type ReactRoleMap = IndexMap<String, id::RoleId>;

#[derive(Debug, Deserialize, Clone)]
pub struct ReactionMapping {
    pub message: serenity::model::id::MessageId,
    pub mapping: ReactRoleMap,
}
