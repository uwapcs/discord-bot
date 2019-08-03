use std::env;

use serenity::{
    model::{channel, channel::Message, gateway::Ready, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

struct Handler;

// #general
const MAIN_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351521117896706);
// #the-corner
const WELCOME_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351613816209418);

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id.0 == 159652921083035648 {
            let mut message = MessageBuilder::new();
            message.push("I see you have seen fit to send another message ");
            message.mention(&msg.author);
            if let Err(why) = msg.channel_id.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg.content == "!vote" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!") {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    fn guild_member_addition(
        &self,
        ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        new_member: Member,
    ) {
        let mut message = MessageBuilder::new();
        message.push("Nice to see you here ");
        message.mention(&new_member);
        message.push("! Would you care to introduce yourself?");
        if let Err(why) = WELCOME_CHANNEL.say(&ctx, message.build()) {
            println!("Error sending message: {:?}", why);
        }

        let mut message = MessageBuilder::new();
        message.push(format!("Say hi to {:?} in ", new_member));
        message.mention(&WELCOME_CHANNEL);
        if let Err(why) = MAIN_CHANNEL.say(&ctx, message.build()) {
            println!("Error sending message: {:?}", why);
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = include_str!("discord_token");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
