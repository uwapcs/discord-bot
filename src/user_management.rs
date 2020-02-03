use rand::Rng;
use serenity::{
    model::{channel::Message, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

use crate::config::CONFIG;
use crate::token_management::*;

macro_rules! e {
    ($error: literal, $x:expr) => {
        match $x {
            Ok(_) => (),
            Err(why) => error!($error, why),
        }
    };
}

pub fn new_member(ctx: &Context, mut new_member: Member) {
    let mut message = MessageBuilder::new();
    message.push("Nice to see you here ");
    message.mention(&new_member);
    message.push_line("! Would you care to introduce yourself?");
    message.push_line("If you're not sure where to start, perhaps you could tell us about your projects, your first computer…");
    message.push_line("You should also know that we follow the Freenode Channel Guidelines: https://freenode.net/changuide, and try to avoid defamatory content");
    if let Err(why) = CONFIG.welcome_channel.say(&ctx, message.build()) {
        error!("Error sending message: {:?}", why);
    }

    let mut message = MessageBuilder::new();
    message.push(format!("Say hi to {} in ", new_member.display_name()));
    message.mention(&CONFIG.welcome_channel);
    if let Err(why) = CONFIG.main_channel.say(&ctx, message.build()) {
        error!("Error sending message: {:?}", why);
    }

    if let Err(why) = new_member.add_role(&ctx.http, CONFIG.unregistered_member_role) {
        error!("Error adding user role: {:?}", why);
    };
}

pub struct Commands;
impl Commands {
    pub fn join(ctx: Context, msg: Message, _content: &str) {
        e!(
            "Unable to get user: {:?}",
            serenity::model::id::GuildId(CONFIG.server_id)
                .member(ctx.http.clone(), msg.author.id)
                .map(|member| new_member(&ctx, member))
        );
    }
    pub fn register(ctx: Context, msg: Message, content: &str) {
        let name = content;
        if name.len() <= 0 {
            e!(
                "Error sending message: {:?}",
                msg.channel_id
                    .say(&ctx.http, "Usage: !register <ucc username>")
            );
            return;
        }
        e!(
            "Error sending message: {:?}",
            // TODO convert to email
            msg.channel_id
                .say(&ctx.http, generate_token(&msg.author, name))
        );
        e!("Error deleting register message: {:?}", msg.delete(ctx));
    }
    pub fn verify(ctx: Context, msg: Message, content: &str) {
        let token = content;
        match parse_token(&msg.author, content) {
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
                                    let mut rng = rand::thread_rng();
                                    m.nickname(format!(
                                        "{}, {}",
                                        name,
                                        [
                                            "The Big Cheese",
                                            "The One and Only",
                                            "The Exalted One",
                                            "not to be trusted",
                                            "The Scoundrel",
                                            "A big fish in a small pond",
                                        ][rng.gen_range(0, 5)]
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
            Err(reason) => e!(
                "Error sending message: {:?}",
                msg.channel_id
                    .say(&ctx.http, format!("Verification error: {:?}", reason))
            ),
        }
        e!("Error deleting register message: {:?}", msg.delete(&ctx));
    }
}
