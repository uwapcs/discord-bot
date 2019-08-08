use serenity::{
    model::{channel, channel::Message, gateway::Ready, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

use rand::Rng;

struct Handler;

static SERVER_ID: u64 = 606351521117896704;
// #general
static MAIN_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351521117896706);
// #the-corner
static WELCOME_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351613816209418);
// #general
static ANNOUNCEMENT_CHANNEL: serenity::model::id::ChannelId =
    serenity::model::id::ChannelId(606351521117896706);

static BOT_ID: u64 = 607078903969742848;

static VOTE_POOL_SIZE: i8 = 2;
static VOTE_ROLE: u64 = 607478818038480937;
static TIEBREAKER_ROLE: u64 = 607509283483025409;
static UNREGISTERED_MEMBER_ROLE: u64 = 608282247350714408;
static REGISTERED_MEMBER_ROLE: u64 = 608282133118582815;

static FOR_VOTE: &str = "👍";
static AGAINST_VOTE: &str = "👎";
static ABSTAIN_VOTE: &str = "🙊";
static ALLOWED_REACTS: &[&'static str] = &[FOR_VOTE, AGAINST_VOTE, ABSTAIN_VOTE];

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
                    "That's quite enough from you ",
                    "Why do you continue to bother us ",
                    "Oh. It's you again ",
                    "What are you doing ",
                ][rng.gen_range(0, 4)],
            );
            message.mention(&msg.author);
            if let Err(why) = msg.channel_id.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg.content.starts_with("!join") {
            match serenity::model::id::GuildId(SERVER_ID).member(ctx.http.clone(), msg.author.id) {
                Ok(member) => {
                    new_member(&ctx, member);
                }
                _ => {}
            }
        }
        if msg.content.starts_with("!move") {
            let mut iter = msg.content.chars();
            iter.by_ref().nth(5);
            let topic = iter.as_str();
            if topic == "" {
                if let Err(why) = msg.channel_id.say(
                    &ctx.http,
                    "If there's something you want to motion, put it after the !move keyword",
                ) {
                    println!("Error sending message: {:?}", why);
                }
            } else {
                create_motion(&ctx, &msg, topic);
            }
        } else if msg.content.starts_with("!motion") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "I hope you're not having a motion. You may have wanted to !move something instead.") {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content.starts_with("!register") {
            let mut iter = msg.content.chars();
            iter.by_ref().nth(9);
            let name = iter.as_str();
            if name == "" {
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "Usage: !register <ucc username>")
                {
                    println!("Error sending message: {:?}", why);
                }
            } else {
                match serenity::model::id::GuildId(SERVER_ID)
                    .member(ctx.http.clone(), msg.author.id)
                {
                    Ok(mut member) => {
                        if let Err(why) = member.remove_role(&ctx.http, UNREGISTERED_MEMBER_ROLE) {
                            println!("Unable to remove role: {:?}", why);
                        };
                        match member.edit(&ctx.http, |m| {
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
                                ][rng.gen_range(0, 4)]
                            ));
                            m
                        }) {
                            Ok(_) => {
                                if let Err(why) = member.add_role(&ctx.http, REGISTERED_MEMBER_ROLE)
                                {
                                    println!("Unable to add role: {:?}", why);
                                };
                            }
                            Err(why) => {
                                println!("Unable to edit nickname: {:?}", why);
                            }
                        };
                    }
                    Err(why) => {
                        println!("Unable to get member: {:?}", why);
                    }
                }
            }
            if let Err(why) = msg.delete(ctx) {
                println!("Error deleting motion prompt: {:?}", why);
            }
        } else if msg.content.starts_with("!cowsay") {
            let mut iter = msg.content.chars();
            iter.by_ref().nth(7);
            let text = iter.as_str();
            let output = std::process::Command::new("cowsay")
                .arg(text)
                .output()
                .expect("failed to execute cowsay");
            let mut message = MessageBuilder::new();
            message.push_codeblock(
                String::from_utf8(output.stdout).expect("unable to parse stdout to String"),
                None,
            );
            if let Err(why) = msg.channel_id.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content.starts_with("!troll") {
            let mut iter = msg.content.chars();
            iter.by_ref().nth(5);
            let text = iter.as_str();
            let output = std::process::Command::new("cat")
                .arg(text)
                .output()
                .expect("failed to execute cowsay");
            let mut message = MessageBuilder::new();
            message.push_codeblock(
                String::from_utf8(output.stdout).expect("unable to parse stdout to String"),
                None,
            );
            if let Err(why) = msg.channel_id.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            }
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
                if message.author.id.0 == BOT_ID {
                    if let Ok(user) = add_reaction.user(&ctx) {
                        match user.has_role(&ctx, SERVER_ID, VOTE_ROLE) {
                            Ok(true) => {
                                for react in [FOR_VOTE, AGAINST_VOTE, ABSTAIN_VOTE]
                                    .iter()
                                    .filter(|r| r != &&add_reaction.emoji.as_data().as_str())
                                {
                                    for a_user in
                                        message.reaction_users(&ctx, *react, None, None).unwrap()
                                    {
                                        if a_user.id.0 == user.id.0 {
                                            if let Err(why) = add_reaction.delete(&ctx) {
                                                println!("Error deleting react: {:?}", why);
                                            };
                                        }
                                    }
                                }
                                if !ALLOWED_REACTS.contains(&add_reaction.emoji.as_data().as_str())
                                {
                                    if let Err(why) = add_reaction.delete(&ctx) {
                                        println!("Error deleting react: {:?}", why);
                                    };
                                }
                                if user.id.0 != BOT_ID {
                                    update_motion(&ctx, &mut message, &user, "add", add_reaction);
                                }
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

    fn reaction_remove(&self, ctx: Context, removed_reaction: channel::Reaction) {
        match removed_reaction.message(&ctx.http) {
            Ok(mut message) => {
                if message.author.id.0 == BOT_ID {
                    if let Ok(user) = removed_reaction.user(&ctx) {
                        update_motion(&ctx, &mut message, &user, "remove", removed_reaction);
                    }
                }
            }
            Err(why) => {
                println!("Error getting user role: {:?}", why);
            }
        }
    }

    fn guild_member_addition(
        &self,
        ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        the_new_member: Member,
    ) {
        new_member(&ctx, the_new_member);
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
    println!("{} created a motion {}", msg.author.name, topic);
    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.colour(serenity::utils::Colour::GOLD);
            embed.title(format!("Motion to {}", topic));
            let mut desc = MessageBuilder::new();
            desc.role(VOTE_ROLE);
            desc.push(" take a look at this motion from ");
            desc.mention(&msg.author);
            embed.description(desc.build());
            embed.field("Status", "Under Consideration", true);
            embed.field("Votes", "For: 0\nAgainst: 0\nAbstain: 0", true);
            embed
        });
        m.reactions(vec![FOR_VOTE, AGAINST_VOTE, ABSTAIN_VOTE]);
        m
    }) {
        Err(why) => {
            println!("Error sending message: {:?}", why);
        }
        Ok(_) => {
            if let Err(why) = msg.delete(ctx) {
                println!("Error deleting motion prompt: {:?}", why);
            }
        }
    }
}

fn update_motion(
    ctx: &Context,
    msg: &mut Message,
    user: &serenity::model::user::User,
    change: &str,
    reaction: channel::Reaction,
) {
    let for_votes = msg.reaction_users(ctx, FOR_VOTE, None, None).unwrap().len() as isize - 1;
    let against_votes = msg
        .reaction_users(ctx, AGAINST_VOTE, None, None)
        .unwrap()
        .len() as isize
        - 1;
    let abstain_votes = msg
        .reaction_users(ctx, ABSTAIN_VOTE, None, None)
        .unwrap()
        .len() as isize
        - 1;

    let strength_buff = |react: &str| {
        msg.reaction_users(ctx, react, None, None)
            .unwrap()
            .iter()
            .filter(|u| match u.has_role(ctx, SERVER_ID, TIEBREAKER_ROLE) {
                Ok(true) => true,
                _ => false,
            })
            .count()
            > 0
    };

    let for_strength = for_votes as f32 + (if strength_buff(FOR_VOTE) { 0.5 } else { 0.0 });
    let against_strength = against_votes as f32
        + (if strength_buff(AGAINST_VOTE) {
            0.5
        } else {
            0.0
        });
    let abstain_strength = abstain_votes as f32
        + (if strength_buff(ABSTAIN_VOTE) {
            0.5
        } else {
            0.0
        });

    let old_embed = msg.embeds[0].clone();
    let topic = old_embed.clone().title.unwrap();

    println!(
        "  {:10} {:6} {} on {}",
        user.name,
        change,
        reaction.emoji.as_data().as_str(),
        topic
    );

    let update_status = |e: &mut serenity::builder::CreateEmbed,
                         status: &str,
                         last_status_full: String,
                         topic: &str| {
        let last_status = last_status_full.lines().next().expect("No previous status");
        if last_status == status {
            e.field("Status", last_status_full, true);
        } else {
            e.field(
                "Status",
                format!("{}\n_was_ {}", status, last_status_full),
                true,
            );
            println!("Motion to {} now {}", topic, status);
            //
            let mut message = MessageBuilder::new();
            message.push_bold(topic);
            message.push(" is now ");
            message.push_bold(status);
            message.push_italic(format!(" (was {})", last_status));
            if let Err(why) = ANNOUNCEMENT_CHANNEL.say(&ctx.http, message.build()) {
                println!("Error sending message: {:?}", why);
            };
        }
    };

    if let Err(why) = msg.edit(ctx, |m| {
        m.embed(|e| {
            e.title(&topic);
            e.description(old_embed.description.unwrap());
            let last_status_full = old_embed
                .fields
                .iter()
                .filter(|f| f.name == "Status")
                .next()
                .expect("No previous status")
                .clone()
                .value;
            if for_strength > (VOTE_POOL_SIZE / 2) as f32 {
                e.colour(serenity::utils::Colour::TEAL);
                update_status(e, "Passed", last_status_full, &topic);
            } else if against_strength + abstain_strength > (VOTE_POOL_SIZE / 2) as f32 {
                e.colour(serenity::utils::Colour::RED);
                update_status(e, "Failed", last_status_full, &topic);
            } else {
                e.colour(serenity::utils::Colour::GOLD);
                update_status(e, "Under Consideration", last_status_full, &topic);
            }
            e.field(
                format!(
                    "Votes ({}/{})",
                    for_votes + against_votes + abstain_votes,
                    VOTE_POOL_SIZE
                ),
                format!(
                    "For: {}\nAgainst: {}\nAbstain: {}",
                    for_votes, against_votes, abstain_votes
                ),
                true,
            );
            e
        })
    }) {
        println!("Error updating motion: {:?}", why);
    }
}

fn new_member(ctx: &Context, mut new_member: Member) {
    let mut message = MessageBuilder::new();
    message.push("Nice to see you here ");
    message.mention(&new_member);
    message.push_line("! Would you care to introduce yourself?");
    message.push_line("If you're not sure where to start, perhaps you could tell us about your projects, your first computer…");
    message.push_line("You should also know that we follow the Freenode Channel Guidelines: https://freenode.net/changuide");
    if let Err(why) = WELCOME_CHANNEL.say(&ctx, message.build()) {
        println!("Error sending message: {:?}", why);
    }

    let mut message = MessageBuilder::new();
    message.push(format!("Say hi to {:?} in ", new_member.display_name()));
    message.mention(&WELCOME_CHANNEL);
    if let Err(why) = MAIN_CHANNEL.say(&ctx, message.build()) {
        println!("Error sending message: {:?}", why);
    }

    if let Err(why) = new_member.add_role(&ctx.http, UNREGISTERED_MEMBER_ROLE) {
        println!("Error adding user role: {:?}", why);
    };
}
