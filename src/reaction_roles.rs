use serenity::{
    model::{channel::Message, channel::Reaction},
    client::Context
};
use crate::util::get_string_from_react;
use crate::config::CONFIG;

pub fn add_role_by_reaction(ctx: Context, msg: Message, added_reaction: Reaction) {
    CONFIG.react_role_messages.iter().find(|rrm| rrm.message == msg.id).and_then(|reaction_mapping| {
        let react_as_string = get_string_from_react(added_reaction.emoji);
        return reaction_mapping.mapping.get(&react_as_string);
    }).and_then(|role_id|{
        return ctx.http.add_member_role(CONFIG.server_id, *msg.author.id.as_u64(), *role_id.as_u64()).ok();
    });
}

pub fn remove_role_by_reaction(ctx: Context, msg: Message, removed_reaction: Reaction) {
    CONFIG.react_role_messages.iter().find(|rrm| rrm.message == msg.id).and_then(|reaction_mapping| {
        let react_as_string = get_string_from_react(removed_reaction.emoji);
        return reaction_mapping.mapping.get(&react_as_string);
    }).and_then(|role_id|{
        return ctx.http.remove_member_role(CONFIG.server_id, *msg.author.id.as_u64(), *role_id.as_u64()).ok();
    });
}
