use serenity::model::channel::ReactionType;

pub fn get_string_from_react(react: ReactionType) -> String {
    match react {
        ReactionType::Custom {animated: _, id: _, name: Some(name)} => name,
        ReactionType::Custom {animated: _, id, name: None} => id.to_string(),
        ReactionType::Unicode(name) => name,
        _ => format!("Unrecognised reaction type: {:?}", react),
    }
}
