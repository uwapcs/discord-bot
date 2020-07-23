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
mod reaction_roles;
mod serenity_handler;
mod token_management;
mod voting;

use config::SECRETS;
use serenity_handler::Handler;

fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("discord-bot.log").unwrap(),
        ),
    ])
    .unwrap();

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&SECRETS.discord_token, Handler).expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
