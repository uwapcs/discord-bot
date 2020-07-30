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
        let messages = match CONFIG
            .readme_channel
            .messages(ctx.http.clone(), |m| m.limit(2))
        {
            Ok(x) if x.len() > 1 => (x[x.len() - 1].clone(), x[x.len() - 2].clone()),
            _ => (
                CONFIG
                    .readme_channel
                    .send_message(ctx.http.clone(), |m| m.content("tmp..."))
                    .unwrap(),
                CONFIG
                    .readme_channel
                    .send_message(ctx.http.clone(), |m| m.content("tmp..."))
                    .unwrap(),
            ),
        };
        welcome_message(&ctx, messages);
        sync_all_role_reactions(&ctx);
    }

    fn resume(&self, ctx: Context, _: serenity::model::event::ResumedEvent) {
        sync_all_role_reactions(&ctx);
    }
}

/// Print a welcome message to a given channel.
/// Requires that there exists two existing messages that can be edited into welcome messages.
/// Has `#[allow(dead_code)]` so it can be commented out without problems
#[allow(dead_code)]
fn welcome_message(ctx: &Context, mut messages: (channel::Message, channel::Message)) {
    // Drop the `Result`
    let _ = messages.0.edit(&ctx.http, |m| {
        use serenity::utils::{EmbedMessageBuilding, MessageBuilder};
        m.embed(|embed| {
            embed.author(|a| {
                a.name("PCS Bot")
                    .url("https://github.com/uwapcs/discord-bot")
            });
            embed.title("Welcome to the PCS Discord!");
            let mut desc = MessageBuilder::new();
            desc.push_named_link("**PCS Website**", "https://pcs.org.au/about");
            desc.push("\n\nYou don't need to be a member to be here (although it definitely helps); so just sit back, relax, and enjoy!\n");
            embed.description(desc.build());
            embed.field("Rules", "We'd appreciate it if you'd follow the Freenode Channel Guidelines\nhttps://freenode.org/changuide", true);
            let mut desc = MessageBuilder::new();
            desc.push_named_link("Constitution", "https://github.com/uwapcs/constitution");
            desc.push_named_link("\nMinutes", "https://github.com/uwapcs/minutes");
            desc.push_named_link("\nBot", "https://github.com/uwapcs/discord-bot");
            embed.field("GitHub", desc.build(), true);
            embed
        });
        m.content(" ")
    });
    // Drop the `Result`
    let _ = messages.1.edit(&ctx.http, |m| {
        m.content("**Permanent discord link**: https://discord.gg/2WaTYnj")
    });
}
