use serenity::{
    model::{channel, gateway::Ready},
    prelude::*,
};

use crate::commands::voting;
use crate::config::CONFIG;
use crate::helpers::*;
use crate::reaction_roles::{
    add_role_by_reaction, remove_role_by_reaction, sync_all_role_reactions,
};

pub struct Handler;

impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, add_reaction: channel::Reaction) {
        match add_reaction.message(&ctx.http) {
            Ok(message) => match get_message_type(&message) {
                MessageType::RoleReactMessage if add_reaction.user_id.0 != CONFIG.bot_id => {
                    add_role_by_reaction(&ctx, message, add_reaction);
                }
                _ if message.author.id.0 != CONFIG.bot_id
                    || add_reaction.user_id == CONFIG.bot_id => {}
                MessageType::Motion => voting::reaction_add(ctx, add_reaction),
                _ => {}
            },
            Err(why) => error!("Failed to get react message {:?}", why),
        }
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: channel::Reaction) {
        match removed_reaction.message(&ctx.http) {
            Ok(message) => match get_message_type(&message) {
                MessageType::RoleReactMessage if removed_reaction.user_id != CONFIG.bot_id => {
                    remove_role_by_reaction(&ctx, message, removed_reaction);
                }
                _ if message.author.id.0 != CONFIG.bot_id
                    || removed_reaction.user_id == CONFIG.bot_id => {}
                MessageType::Motion => voting::reaction_remove(ctx, removed_reaction),
                _ => {}
            },
            Err(why) => error!("Failed to get react message {:?}", why),
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        sync_all_role_reactions(&ctx);
    }

    fn resume(&self, ctx: Context, _: serenity::model::event::ResumedEvent) {
        sync_all_role_reactions(&ctx);
    }
}
