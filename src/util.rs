use serenity::model::{channel::ReactionType, guild::PartialGuild};

pub fn get_string_from_react(react: &ReactionType) -> String {
    match react {
        ReactionType::Custom {
            name: Some(name), ..
        } => name.to_string(),
        ReactionType::Custom { id, name: None, .. } => id.to_string(),
        ReactionType::Unicode(name) => name.to_string(),
        _ => format!("Unrecognised reaction type: {:?}", react),
    }
}

pub fn get_react_from_string(string: String, guild: PartialGuild) -> ReactionType {
    guild
        .emojis
        .values()
        .find(|e| e.name == string)
        .map_or_else(
            || ReactionType::from(string), // unicode emoji
            |custom_emoji| ReactionType::from(custom_emoji.clone()),
        )
}

#[macro_use]
macro_rules! e {
    ($error: literal, $x:expr) => {
        match $x {
            Ok(_) => (),
            Err(why) => error!($error, why),
        }
    };
}

#[macro_use]
macro_rules! send_message {
    ($chan:expr, $context:expr, $message:expr) => {
        $chan.say($context, $message).map_err(|why| {
            error!("Error sending message: {:?}", why);
            why
        })
    };
}

#[allow(unused_macros)] // remove this if you start using it
#[macro_use]
macro_rules! send_delete_message {
    ($chan:expr, $context:expr, $message:expr) => {
        match $chan.say($context, $message) {
            Ok(the_new_msg) => e!(
                "Error deleting register message: {:?}",
                the_new_msg.delete($context)
            ),
            Err(why) => error!("Error sending message: {:?}", why),
        }
    };
}
