#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate indexmap;
extern crate simplelog;
#[macro_use]
extern crate guard;

extern crate reqwest;

use simplelog::*;
use std::fs::File;

use serenity::client::Client;

#[macro_use]
mod util;
mod config;
mod commands {
    pub mod fun;
    pub mod general;
    pub mod voting;
}
mod helpers;
mod reaction_roles;
mod serenity_handler;

use config::SECRETS;
use serenity_handler::Handler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).map_or_else(
            || SimpleLogger::new(LevelFilter::Info, Config::default()) as Box<dyn SharedLogger>,
            |x| x as Box<dyn SharedLogger>,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("discord-bot.log").unwrap(),
        ),
    ])?;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&SECRETS.discord_token, Handler)?;

    client.with_framework(
        serenity::framework::StandardFramework::new()
            .configure(|c| c.prefix(&config::CONFIG.command_prefix))
            .unrecognised_command(|ctx, msg, command_name| {
                warn!(
                    "User {}#{:4} tried to call non-existant command {}",
                    msg.author.name, msg.author.discriminator, command_name
                );
                send_message!(msg.channel_id, &ctx.http, "Unrecognised command. Try !help")
                    .map(|_| ())
                    .or(<Result<(), &dyn std::error::Error>>::Ok(()))
                    .unwrap();
            })
            .prefix_only(|ctx, msg| helpers::sassy(ctx, msg))
            .normal_message(|ctx, msg| {
                if msg
                    .content
                    .contains(&format!("<@!{}>", config::CONFIG.bot_id))
                    || msg
                        .content
                        .contains(&format!("<@{}>", config::CONFIG.bot_id))
                {
                    helpers::sassy(ctx, msg)
                }
            })
            .group(&commands::general::GENERAL_GROUP)
            .group(&commands::voting::VOTING_GROUP)
            .group(&commands::fun::FUN_GROUP)
            .help(&helpers::HELP),
    );

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    Ok(client.start_autosharded()?)
}
