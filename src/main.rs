#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate indexmap;
extern crate simplelog;
#[macro_use]
extern crate guard;
use simplelog::*;
use std::fs::{read_to_string, File};

use chrono::prelude::Utc;
use serenity::{
    model::{channel, channel::Message, gateway::Ready, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

mod config;
mod reaction_roles;
mod token_management;
mod user_management;
mod util;
mod voting;

use config::CONFIG;
use reaction_roles::{add_role_by_reaction, remove_role_by_reaction};
use util::get_string_from_react;

macro_rules! e {
    ($error: literal, $x:expr) => {
        match $x {
            Ok(_) => (),
            Err(why) => error!($error, why),
        }
    };
}

struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if !(msg.content.starts_with(&CONFIG.command_prefix)) {
            return;
        }
        let message_content: Vec<_> = msg.content[1..].splitn(2, ' ').collect();
        match message_content[0] {
            "say" => {
                println!("{:#?}", msg.content);
            }
            "register" => user_management::Commands::register(ctx, msg.clone(), message_content[1]),
            "verify" => user_management::Commands::verify(ctx, msg.clone(), message_content[1]),
            "move" => {
                voting::Commands::move_something(ctx, msg.clone(), message_content[1]);
            }
            "motion" => {
                voting::Commands::motion(ctx, msg.clone(), message_content[1]);
            }
            "poll" => {
                voting::Commands::poll(ctx, msg.clone(), message_content[1]);
            }
            "cowsay" => {
                voting::Commands::cowsay(ctx, msg.clone(), message_content[1]);
            }
            "logreact" => {
                e!("Error deleting logreact prompt: {:?}", msg.delete(&ctx));
                e!(
                    "Error sending message {:?}",
                    msg.channel_id
                        .say(&ctx.http, "React to this to log the ID (for the next 5min)")
                )
            }
            "help" => {
                let mut message = MessageBuilder::new();
                message.push_line(format!(
                    "Use {}move <action> to make a circular motion",
                    &CONFIG.command_prefix
                ));
                message.push_line(format!(
                    "Use {}poll <proposal> to see what people think about something",
                    &CONFIG.command_prefix
                ));
                e!(
                    "Error sending message: {:?}",
                    msg.channel_id.say(&ctx.http, message.build())
                );
            }
            _ => {
                e!(
                    "Error sending message: {:?}",
                    msg.channel_id.say(
                        &ctx.http,
                        format!("Unrecognised command. Try {}help", &CONFIG.command_prefix)
                    )
                );
            }
        }
    }

    fn reaction_add(&self, ctx: Context, add_reaction: channel::Reaction) {
        match add_reaction.message(&ctx.http) {
            Ok(message) => {
                let message_type = get_message_type(&message);
                if message_type == MessageType::RoleReactMessage
                    && add_reaction.user_id.0 != CONFIG.bot_id
                {
                    add_role_by_reaction(&ctx, message, add_reaction);
                    return;
                }
                if message.author.id.0 != CONFIG.bot_id || add_reaction.user_id == CONFIG.bot_id {
                    return;
                }
                match message_type {
                    MessageType::Motion => {
                        voting::reaction_add(ctx, add_reaction);
                    }
                    MessageType::LogReact => {
                        let react_user = add_reaction.user(&ctx).unwrap();
                        let react_as_string = get_string_from_react(&add_reaction.emoji);
                        if Utc::now().timestamp() - message.timestamp.timestamp() > 300 {
                            warn!(
                                "The logreact message {} just tried to use is too old",
                                react_user.name
                            );
                            return;
                        }
                        info!(
                            "The react {} just added is {:?}. In full: {:?}",
                            react_user.name, react_as_string, add_reaction.emoji
                        );
                        let mut msg = MessageBuilder::new();
                        msg.push_italic(react_user.name);
                        msg.push(format!(
                            " wanted to know that {} is represented by ",
                            add_reaction.emoji,
                        ));
                        msg.push_mono(react_as_string);
                        e!(
                            "Error sending message: {:?}",
                            message.channel_id.say(&ctx.http, msg.build())
                        );
                    }
                    _ => {}
                }
            }
            Err(why) => error!("Failed to get react message {:?}", why),
        }
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: channel::Reaction) {
        match removed_reaction.message(&ctx.http) {
            Ok(message) => {
                let message_type = get_message_type(&message);
                if message_type == MessageType::RoleReactMessage
                    && removed_reaction.user_id != CONFIG.bot_id
                {
                    remove_role_by_reaction(&ctx, message, removed_reaction);
                    return;
                }
                if message.author.id.0 != CONFIG.bot_id || removed_reaction.user_id == CONFIG.bot_id
                {
                    return;
                }
                if message_type == MessageType::Motion {
                    voting::reaction_remove(ctx, removed_reaction);
                }
            }
            Err(why) => error!("Failed to get react message {:?}", why),
        }
    }

    fn guild_member_addition(
        &self,
        ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        the_new_member: Member,
    ) {
        user_management::new_member(&ctx, the_new_member);
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        reaction_roles::sync_all_role_reactions(&ctx);
    }

    fn resume(&self, ctx: Context, _: serenity::model::event::ResumedEvent) {
        reaction_roles::sync_all_role_reactions(&ctx);
    }
}

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("ucc-bot.log").unwrap(),
        ),
    ])
    .unwrap();

    // Configure the client with your Discord bot token in the environment.
    let token = read_to_string("discord_token").unwrap();

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

#[derive(Debug, PartialEq)]
enum MessageType {
    Motion,
    Role,
    RoleReactMessage,
    LogReact,
    Poll,
    Misc,
}

fn get_message_type(message: &Message) -> MessageType {
    if CONFIG
        .react_role_messages
        .iter()
        .any(|rrm| rrm.message == message.id)
    {
        return MessageType::RoleReactMessage;
    }
    if message.embeds.is_empty() {
        // Get first word of message
        return match message.content.splitn(2, ' ').next().unwrap() {
            "Role" => MessageType::Role,
            "React" => MessageType::LogReact,
            _ => MessageType::Misc,
        };
    }
    let title: String = message.embeds[0].title.clone().unwrap();
    let words_of_title: Vec<_> = title.splitn(2, ' ').collect();
    let first_word_of_title = words_of_title[0];
    match first_word_of_title {
        "Motion" => MessageType::Motion,
        "Poll" => MessageType::Poll,
        _ => MessageType::Misc,
    }
}
