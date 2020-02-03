use serenity::model::{channel::ReactionType, guild::PartialGuild};

pub fn get_string_from_react(react: ReactionType) -> String {
    match react {
        ReactionType::Custom {animated: _, id: _, name: Some(name)} => name,
        ReactionType::Custom {animated: _, id, name: None} => id.to_string(),
        ReactionType::Unicode(name) => name,
        _ => format!("Unrecognised reaction type: {:?}", react),
    }
}

pub fn get_react_from_string(string: String, guild: PartialGuild) -> ReactionType {
     guild.emojis.values().find(|e| e.name == string).map_or_else(
         || ReactionType::from(string), // unicode emoji
         |custom_emoji| ReactionType::from(custom_emoji.id))
}
