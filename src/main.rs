#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::*;
use std::fs::File;

use serenity::{
    model::{channel, channel::Message, gateway::Ready, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

mod config;
mod user_management;
mod voting;

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
        if !(msg.content.starts_with(config::COMMAND_PREFIX)) {
            return;
        }
        let message_content: Vec<_> = msg.content[1..].splitn(2, ' ').collect();
        match message_content[0] {
            "say" => {
                println!("{:#?}", msg.content);
            }
            "register" => user_management::Commands::register(ctx, msg.clone(), message_content[1]),
            "join" => {
                user_management::Commands::join(ctx, msg.clone(), message_content[1]);
            }
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
            "help" => {
                let mut message = MessageBuilder::new();
                message.push_line(format!(
                    "Use {}move <action> to make a circular motion",
                    config::COMMAND_PREFIX
                ));
                message.push_line(format!(
                    "Use {}poll <proposal> to see what people think about something",
                    config::COMMAND_PREFIX
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
                        format!("Unrecognised command. Try {}help", config::COMMAND_PREFIX)
                    )
                );
            }
        }
    }

    fn reaction_add(&self, ctx: Context, add_reaction: channel::Reaction) {
        match add_reaction.message(&ctx.http) {
            Ok(message) => {
                if message.author.id.0 != config::BOT_ID || add_reaction.user_id == config::BOT_ID {
                    return;
                }
                match message_type(&message) {
                    "motion" => {
                        voting::reaction_add(ctx, add_reaction);
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
                if message.author.id.0 != config::BOT_ID || removed_reaction.user_id == config::BOT_ID {
                    return;
                }
                match message_type(&message) {
                    "motion" => {
                        voting::reaction_remove(ctx, removed_reaction);
                    }
                    _ => {}
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
    fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
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
    let token = config::DISCORD_TOKEN;

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

fn message_type(message: &Message) -> &'static str {
    if message.embeds.len() <= 0 {
        return match message.content.splitn(2, ' ').next().unwrap() {
            "Role" => "role",
            _ => "misc",
        };
    }
    let title: String = message.embeds[0].title.clone().unwrap();
    let words_of_title: Vec<_> = title.splitn(2, ' ').collect();
    let first_word_of_title = words_of_title[0];
    return match first_word_of_title {
        "Motion" => "motion",
        "Poll" => "poll",
        _ => "misc",
    };
}
