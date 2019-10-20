use serenity;

pub static DISCORD_TOKEN: &str = include_str!("discord_token");

pub static SERVER_ID: u64 = 264401248676085760;
// #ucc
pub static MAIN_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(264401248676085760);
// #welcome
pub static WELCOME_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606750983699300372);
// #committee
pub static ANNOUNCEMENT_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(264411219627212801);

pub static BOT_ID: u64 = 607078903969742848;

pub static VOTE_POOL_SIZE: i8 = 7;
pub static VOTE_ROLE: u64 = 269817189966544896;
pub static TIEBREAKER_ROLE: u64 = 635370432568098817;
pub static UNREGISTERED_MEMBER_ROLE: u64 = 0; // does not exist
pub static REGISTERED_MEMBER_ROLE: u64 = 0; // does not exist

pub static COMMAND_PREFIX: &str = "!";

pub static FOR_VOTE: &str = "👍";
pub static AGAINST_VOTE: &str = "👎";
pub static ABSTAIN_VOTE: &str = "🙊";
pub static APPROVE_REACT: &str = "⬆";
pub static DISAPPROVE_REACT: &str = "⬇";
pub static UNSURE_REACT: &str = "❔";
pub static ALLOWED_REACTS: &[&'static str] = &[
    FOR_VOTE,
    AGAINST_VOTE,
    ABSTAIN_VOTE,
    APPROVE_REACT,
    DISAPPROVE_REACT,
    UNSURE_REACT,
];
