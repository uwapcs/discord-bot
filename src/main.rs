use serenity::{
    model::{channel, channel::Message, gateway::Ready, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

extern crate rand;

use rand::Rng;

struct Handler;

// #general
const MAIN_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351521117896706);
// #the-corner
const WELCOME_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351613816209418);

const BOT_ID: u64 = 607078903969742848;

const VOTE_ROLE: u64 = 607478818038480937;

const SERVER_ID: u64 = 606351521117896704;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id.0 == 159652921083035648 {
            let mut rng = rand::thread_rng();
            let mut message = MessageBuilder::new();
            message.push(
                [
                    "I see you have seen fit to send another message ",
                    "Why do you continue to bother us ",
                    "Oh. It's you again ",
                    "What are you doing ",
                ][rng.gen_range(0, 3)],
            );
            message.mention(&msg.author);
            if let Err(why) = msg.channel_id.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg.content.starts_with("!move") {
            let mut iter = msg.content.chars();
            iter.by_ref().nth(5);
            let topic = iter.as_str();
            create_motion(&ctx, &msg, topic);
        } else if msg.content.starts_with("!motion") {
            let mut iter = msg.content.chars();
            iter.by_ref().nth(7);
            let topic = iter.as_str();
            create_motion(&ctx, &msg, topic);
        } else if msg.content == "!help" {
            let mut message = MessageBuilder::new();
            message.push("Use !move <action> to make a circular motion");
            if let Err(why) = msg.channel_id.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    fn reaction_add(&self, ctx: Context, add_reaction: channel::Reaction) {
        match add_reaction.message(&ctx.http) {
            Ok(mut message) => {
                println!("{:#?}", message.embeds[0]);
                if message.author.id.0 == BOT_ID {
                    if let Ok(user) = add_reaction.user(&ctx) {
                        match user.has_role(&ctx, SERVER_ID, VOTE_ROLE) {
                            Ok(true) => {
                                // for reaction in message.reactions {
                                //     // FIXME: this isn't right
                                //     if reaction.me {
                                //         if let Err(why) = add_reaction.delete(&ctx) {
                                //             println!("Error deleting react: {:?}", why);
                                //         };
                                //     }
                                // }
                                updateMotion(&ctx, &mut message, &user);
                            }
                            Ok(false) => {
                                if user.id.0 != BOT_ID {
                                    if let Err(why) = add_reaction.delete(&ctx) {
                                        println!("Error deleting react: {:?}", why);
                                    };
                                }
                            }
                            Err(why) => {
                                println!("Error getting user role: {:?}", why);
                            }
                        }
                    }
                }
            }
            Err(why) => {
                println!("Error processing react: {:?}", why);
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
        message.push(format!("Say hi to {:?} in ", new_member.display_name()));
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

fn create_motion(ctx: &Context, msg: &Message, topic: &str) {
    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.colour(serenity::utils::Colour::GOLD);
            embed.title(format!("Motion to {}", topic));
            let mut desc = MessageBuilder::new();
            desc.push("Motion by ");
            desc.mention(&msg.author);
            embed.description(desc.build());
            embed.field("Status", "Under Consideration", true);
            embed.field(
                "Votes",
                "👍 For: ?\n👎 Against: ?\n🙊 Abstain: ?",
                true,
            );
            embed.footer(|f| {
                f.text("Motion power: 0");
                f
            });
            embed
        });
        m
    }) {
        Err(why) => {
            println!("Error sending message: {:?}", why);
        }
        Ok(message) => {
            if let Err(why) = msg.delete(ctx) {
                println!("Error deleting motion prompt: {:?}", why);
            }
            if let Err(why) = message.react(ctx, "👍") {
                println!("Error sending 👍 react: {:?}", why);
            }
            if let Err(why) = message.react(ctx, "👎") {
                println!("Error sending 👎 react: {:?}", why);
            }
            if let Err(why) = message.react(ctx, "🙊") {
                println!("Error sending 🤷 react: {:?}", why);
            }
        }
    }
}

fn updateMotion(ctx: &Context, msg: &mut Message, user: &serenity::model::user::User) {
    let old_embed = msg.embeds[0].clone();
    if let Err(why) = msg.edit(ctx, |m| {
        m.embed(|e| {
            e.title(old_embed.title.unwrap());
            e.colour(serenity::utils::Colour::RED);
            e.description(old_embed.description.unwrap());
            e
        })
    }) {
        println!("Error updating motion: {:?}", why);
    }
}
