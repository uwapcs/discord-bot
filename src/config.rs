use serenity;

pub static DISCORD_TOKEN: &str = include_str!("discord_token");

pub static SERVER_ID: u64 = 606351521117896704;
// #general
pub static MAIN_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351521117896706);
// #the-corner
pub static WELCOME_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351613816209418);
// #general
pub static ANNOUNCEMENT_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351521117896706);

pub static BOT_ID: u64 = 607078903969742848;

pub static VOTE_POOL_SIZE: i8 = 2;
pub static VOTE_ROLE: u64 = 607478818038480937;
pub static TIEBREAKER_ROLE: u64 = 607509283483025409;
pub static UNREGISTERED_MEMBER_ROLE: u64 = 608282247350714408;
pub static REGISTERED_MEMBER_ROLE: u64 = 608282133118582815;

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
