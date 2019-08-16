use rand::Rng;
use serenity::{
    model::{channel::Message, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

use crate::config;

macro_rules! e {
    ($error: literal, $x:expr) => {
        match $x {
            Ok(_) => (),
            Err(why) => eprintln!($error, why),
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
    if let Err(why) = config::WELCOME_CHANNEL.say(&ctx, message.build()) {
        println!("Error sending message: {:?}", why);
    }

    let mut message = MessageBuilder::new();
    message.push(format!("Say hi to {} in ", new_member.display_name()));
    message.mention(&config::WELCOME_CHANNEL);
    if let Err(why) = config::MAIN_CHANNEL.say(&ctx, message.build()) {
        println!("Error sending message: {:?}", why);
    }

    if let Err(why) = new_member.add_role(&ctx.http, config::UNREGISTERED_MEMBER_ROLE) {
        println!("Error adding user role: {:?}", why);
    };
}

pub struct Commands;
impl Commands {
    pub fn join(ctx: Context, msg: Message, _content: &str) {
        e!(
            "Unable to get user: {:?}",
            serenity::model::id::GuildId(config::SERVER_ID)
                .member(ctx.http.clone(), msg.author.id)
                .map(|member| new_member(&ctx, member))
        );
    }
    pub fn register(ctx: Context, msg: Message, content: &str) {
        let name = content;
        if name.len() > 0 {
            e!(
                "Unable to get member: {:?}",
                serenity::model::id::GuildId(config::SERVER_ID)
                    .member(ctx.http.clone(), msg.author.id)
                    .map(|mut member| {
                        e!(
                            "Unable to remove role: {:?}",
                            member.remove_role(&ctx.http, config::UNREGISTERED_MEMBER_ROLE)
                        );
                        e!(
                            "Unable to edit nickname: {:?}",
                            member
                                .edit(&ctx.http, |m| {
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
                                .map(|()| {
                                    e!(
                                        "Unable to add role: {:?}",
                                        member.add_role(&ctx.http, config::REGISTERED_MEMBER_ROLE)
                                    );
                                })
                        );
                    })
            );
            e!("Error deleting register message: {:?}", msg.delete(ctx));
        } else {
            e!(
                "Error sending message: {:?}",
                msg.channel_id
                    .say(&ctx.http, "Usage: !register <ucc username>")
            );
        }
    }
}
