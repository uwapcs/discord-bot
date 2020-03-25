use crate::config::{ReactRoleMap, CONFIG};
use crate::util::{get_react_from_string, get_string_from_react};
use rayon::prelude::*;
use serenity::{
    client::Context,
    model::{channel::Message, channel::Reaction, id::RoleId, id::UserId},
};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub fn add_role_by_reaction(ctx: &Context, msg: Message, added_reaction: Reaction) {
    let user = added_reaction
        .user_id
        .to_user(ctx)
        .expect("Unable to get user");
    if let Some(role_id) = CONFIG
        .react_role_messages
        .iter()
        .find(|rrm| rrm.message == msg.id)
        .and_then(|reaction_mapping| {
            let react_as_string = get_string_from_react(&added_reaction.emoji);
            reaction_mapping.mapping.get(&react_as_string)
        })
    {
        info!(
            "{} requested role '{}'",
            user.name,
            role_id
                .to_role_cached(ctx)
                .expect("Unable to get role")
                .name
        );
        ctx.http
            .add_member_role(
                CONFIG.server_id,
                added_reaction.user_id.0,
                *role_id.as_u64(),
            )
            .ok();
    } else {
        warn!("{} provided invalid react for role", user.name);
        e!("Unable to delete react: {:?}", added_reaction.delete(ctx));
    }
}

pub fn remove_role_by_reaction(ctx: &Context, msg: Message, removed_reaction: Reaction) {
    CONFIG
        .react_role_messages
        .iter()
        .find(|rrm| rrm.message == msg.id)
        .and_then(|reaction_mapping| {
            let react_as_string = get_string_from_react(&removed_reaction.emoji);
            reaction_mapping.mapping.get(&react_as_string)
        })
        .and_then(|role_id| {
            info!(
                "{} requested removal of role '{}'",
                msg.author.name,
                role_id
                    .to_role_cached(ctx)
                    .expect("Unable to get role")
                    .name
            );
            ctx.http
                .remove_member_role(
                    CONFIG.server_id,
                    removed_reaction.user_id.0,
                    *role_id.as_u64(),
                )
                .ok()
        });
}

pub fn sync_all_role_reactions(ctx: &Context) {
    info!("Syncing roles to reactions");
    let messages_with_role_mappings = get_all_role_reaction_message(ctx);
    info!("  Sync: reaction messages fetched");
    let guild = ctx.http.get_guild(CONFIG.server_id).unwrap();
    info!("  Sync: guild fetched");
    // this method supports paging, but we probably don't need it since the server only has a couple of
    // hundred members. the Reaction.users() method can apparently only retrieve 100 users at once, but
    // this one seems to work fine when set to 1000 (I tried 10,000 but the api returned a 400)
    let mut all_members = ctx
        .http
        .get_guild_members(CONFIG.server_id, Some(1000), None)
        .unwrap();
    all_members.retain(|m| m.user_id() != CONFIG.bot_id);
    info!("  Sync: all members fetched");

    let mut roles_to_add: HashMap<UserId, Vec<RoleId>> =
        HashMap::from_iter(all_members.iter().map(|m| (m.user_id(), Vec::new())));
    let mut roles_to_remove: HashMap<UserId, Vec<RoleId>> =
        HashMap::from_iter(all_members.iter().map(|m| (m.user_id(), Vec::new())));

    for (i, (message, mapping)) in messages_with_role_mappings.iter().enumerate() {
        info!("  Sync: prossessing message #{}", i);
        for react in &message.reactions {
            let react_as_string = get_string_from_react(&react.reaction_type);
            if mapping.contains_key(&react_as_string) {
                continue;
            }
            info!(
                "    message #{}: Removing non-role react '{}'",
                i, react_as_string
            );
            for _illegal_react in
                &message.reaction_users(ctx, react.reaction_type.clone(), Some(100), None)
            {
                warn!("    need to implement react removal");
            }
        }
        for (react, role) in *mapping {
            info!("    message #{}: processing react '{}'", i, react);
            // TODO: proper pagination for the unlikely scenario that there are more than 100 (255?) reactions?
            let reaction_type = get_react_from_string(react.clone(), guild.clone());
            let reactors = message
                .reaction_users(ctx.http.clone(), reaction_type.clone(), Some(100), None)
                .unwrap();
            let reactor_ids: HashSet<UserId> = HashSet::from_iter(reactors.iter().map(|r| r.id));

            // ensure bot has reacted
            if !reactor_ids.contains(&UserId::from(CONFIG.bot_id)) {
                e!(
                    "Unable to add reaction, {:?}",
                    message.react(ctx, reaction_type)
                );
            }

            for member in all_members.clone() {
                let user_id = &member.user_id();
                if reactor_ids.contains(&user_id) {
                    if !member.roles.iter().any(|r| r == role) {
                        roles_to_add.get_mut(&user_id).unwrap().push(*role);
                    }
                } else if member.roles.iter().any(|r| r == role) {
                    roles_to_remove.get_mut(&user_id).unwrap().push(*role);
                }
            }
        }
    }
    info!("  Sync: finished determing roles to add/remove");

    for (user_id, roles) in roles_to_add {
        if !roles.is_empty() {
            let mut member = all_members
                .iter()
                .find(|m| m.user_id() == user_id)
                .unwrap()
                .clone();
            member.add_roles(ctx.http.clone(), &roles[..]).unwrap();
        }
    }
    info!("  Sync: (any) missing roles added");
    for (user_id, roles) in roles_to_remove {
        if !roles.is_empty() {
            let mut member = all_members
                .iter()
                .find(|m| m.user_id() == user_id)
                .unwrap()
                .clone();
            member.remove_roles(ctx.http.clone(), &roles[..]).unwrap();
        }
    }
    info!("  Sync: (any) superflous roles removed");
    info!("Role reaction sync complete");
}

fn get_all_role_reaction_message(ctx: &Context) -> Vec<(Message, &'static ReactRoleMap)> {
    let guild = ctx.http.get_guild(CONFIG.server_id).unwrap();
    info!("  Find role-react message: guild determined");
    let channels = ctx.http.get_channels(*guild.id.as_u64()).unwrap();
    info!("  Find role-react message: channels determined");
    let http = ctx.http.clone();
    channels
        .par_iter()
        .flat_map(|channel| {
            // since we don't know which channels the messages are in, we check every combination
            // of message and channel and ignore the bad matches using .ok() and .filter_map()
            let h = http.clone(); // thread-local copy
            CONFIG
                .react_role_messages
                .par_iter()
                .filter_map(move |rrm| {
                    h.get_message(*channel.id.as_u64(), *rrm.message.as_u64())
                        .ok()
                        .map(|m| (m, &rrm.mapping))
                })
        })
        .collect()
}
