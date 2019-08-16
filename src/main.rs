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
            Err(why) => eprintln!($error, why),
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
        if msg.content.starts_with(config::COMMAND_PREFIX) {
            let message_content: Vec<_> = msg.content[1..].splitn(2, ' ').collect();
            match message_content[0] {
                "register" => {
                    user_management::Commands::register(ctx, msg.clone(), message_content[1])
                }
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
    }

    fn reaction_add(&self, ctx: Context, add_reaction: channel::Reaction) {
        voting::reaction_add(ctx, add_reaction);
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: channel::Reaction) {
        voting::reaction_remove(ctx, removed_reaction);
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
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
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
        println!("Client error: {:?}", why);
    }
}
