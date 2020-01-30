use serenity;

pub const CONFIG: UccbotConfig = UccbotConfig {
    discord_token: include_str!("discord_token"),
    server_id: 606351521117896704,
    main_channel: serenity::model::id::ChannelId(606351521117896706),
    welcome_channel: serenity::model::id::ChannelId(606351613816209418),
    announcement_channel: serenity::model::id::ChannelId(606351521117896706),
    bot_id: 607078903969742848,
    vote_pool_size: 2,
    vote_role: 607478818038480937,
    tiebreaker_role: 607509283483025409,
    unregistered_member_role: 608282247350714408,
    registered_member_role: 608282133118582815,
    command_prefix: "!",
    for_vote: "👍",
    against_vote: "👎",
    abstain_vote: "🙊",
    approve_react: "⬆",
    disapprove_react: "⬇",
    unsure_react: "❔",
};

pub struct UccbotConfig {
    pub discord_token: &'static str,
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