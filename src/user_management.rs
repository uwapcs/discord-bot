use rand::seq::SliceRandom;
use serenity::{
    model::{channel::Message, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

use crate::config::CONFIG;
use crate::token_management::*;

pub fn new_member(ctx: &Context, mut new_member: Member) {
    let mut message = MessageBuilder::new();
    message.push("Nice to see you here ");
    message.mention(&new_member);
    message.push_line("! Would you care to introduce yourself?");
    message.push_line("If you're not sure where to start, perhaps you could tell us about your projects, your first computer…");
    message.push_line("You should also know that we follow the Freenode Channel Guidelines: https://freenode.net/changuide, and try to avoid defamatory content");
    send_message!(CONFIG.welcome_channel, &ctx, message.build());

    let mut message = MessageBuilder::new();
    message.push(format!("Say hi to {} in ", new_member.display_name()));
    message.mention(&CONFIG.welcome_channel);
    send_message!(CONFIG.main_channel, &ctx, message.build());

    if let Err(why) = new_member.add_role(&ctx.http, CONFIG.unregistered_member_role) {
        error!("Error adding user role: {:?}", why);
    };
}

pub const RANDOM_NICKNAMES: &[&str] = &[
    "The Big Cheese",
    "The One and Only",
    "The Exalted One",
    "not to be trusted",
    "The Scoundrel",
    "A big fish in a small pond",
];

pub struct Commands;
impl Commands {
    pub fn register(ctx: Context, msg: Message, account_name: &str) {
        if account_name.is_empty() {
            send_message!(msg.channel_id, &ctx.http, "Usage: !register <ucc username>");
            return;
        }
        send_message!(
            msg.channel_id, 
            &ctx.http, 
            format!("Hey {} here's that token you ordered: {}\nIf this wasn't you just ignore this.", 
                account_name, 
                generate_token(&msg.author, account_name)));
        e!("Error deleting register message: {:?}", msg.delete(ctx));
    }
    pub fn verify(ctx: Context, msg: Message, token: &str) {
        match parse_token(&msg.author, token) {
            Ok(name) => {
                e!(
                    "Unable to get member: {:?}",
                    serenity::model::id::GuildId(CONFIG.server_id)
                        .member(ctx.http.clone(), msg.author.id)
                        .map(|mut member| {
                            e!(
                                "Unable to remove role: {:?}",
                                member.remove_role(&ctx.http, CONFIG.unregistered_member_role)
                            );
                            e!(
                                "Unable to edit nickname: {:?}",
                                member.edit(&ctx.http, |m| {
                                    m.nickname(format!(
                                        "{}, {:?}",
                                        name,
                                        RANDOM_NICKNAMES.choose(&mut rand::thread_rng())
                                    ));
                                    m
                                })
                            );
                            let new_msg = msg
                                .channel_id
                                .say(&ctx.http, "Verification succesful")
                                .expect("Error sending message");
                            e!(
                                "Error deleting register message: {:?}",
                                new_msg.delete(&ctx)
                            );
                        })
                );
            }
            Err(reason) => send_message!(msg.channel_id, &ctx.http, format!("Verification error: {:?}", reason)),
        }
        e!("Error deleting register message: {:?}", msg.delete(&ctx));
    }
}
