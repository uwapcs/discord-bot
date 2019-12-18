
#[derive(Debug, Clone)]
struct ReactionMapping {
    mapping: HashMap<serenity::model::id::EmojiId, serenity::model::id::RoleId>,
}

lazy_static! {
    static ref REACTIONS_CACHE: Mutex<HashMap<serenity::model::id::MessageId, ReactionMapping>> =
        Mutex::new(HashMap::new());
}
