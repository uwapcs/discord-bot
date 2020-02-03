use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use serenity::{
    model::{channel::Message, channel::Reaction, id::UserId},
    client::Context
};
use crate::util::{get_string_from_react, get_react_from_string};
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

pub fn add_all_role_reactions(ctx: Context) {
    let messages_with_role_mappings = get_all_role_reaction_message(&ctx);
    let guild = ctx.http.get_guild(CONFIG.server_id).unwrap();
    // this method supports paging, but we probably don't need it since the server only has a couple of
    // hundred members. the Reaction.users() method can apparently only retrieve 100 users at once, but
    // this one seems to work fine when set to 1000 (I tried 10,000 but the api returned a 400)
    let all_members = ctx.http.get_guild_members(CONFIG.server_id, Some(1000), None).unwrap();

    for (message, mapping) in messages_with_role_mappings {
        for (react, role) in mapping {
            // the docs say this method can't retrieve more than 100 user reactions at a time, but it seems
            // to work fine when set to 255...
            // TODO: proper pagination for the unlikely scenario that there are more than 100 (255?) reactions?
            let reaction_type = get_react_from_string(react.clone(), guild.clone());
            let reactors = message.reaction_users(ctx.http.clone(), reaction_type, Some(255), None).unwrap();
            let reactor_ids: HashSet<UserId> = HashSet::from_iter(reactors.iter().map(|r| r.id));

            // this looks O(n!), but n will probably never be more than three digits, so maybe it's okay?
            // one solution might be to batch up all the roles to add/remove for each member and do them
            // all at once with .add_roles()
            for mut member in all_members.clone() {
                if reactor_ids.contains(&member.user_id()) {
                    member.add_role(ctx.http.clone(), role).unwrap();
                } else {
                    member.remove_role(ctx.http.clone(), role).unwrap();
                }
            }
        }
    }
}

fn get_all_role_reaction_message(ctx: &Context) -> Vec<(Message, &'static HashMap<String, serenity::model::id::RoleId>)> {
    let guild = ctx.http.get_guild(CONFIG.server_id).unwrap();
    let channels = ctx.http.get_channels(*guild.id.as_u64()).unwrap();
    return channels.iter().flat_map(|channel| {
        let ctxx = ctx.clone();
        // since we don't know which channels the messages are in, we check every combination of message and
        // channel and ignore the bad matches using .ok() and .filter_map()
        CONFIG.react_role_messages.iter().filter_map(move |rrm|
            ctxx.http.get_message(*channel.id.as_u64(), *rrm.message.as_u64()).ok().map(|m| (m, &rrm.mapping)))
    }).collect();
}
