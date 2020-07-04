use chrono::prelude::Utc;
use serenity::{
    model::{channel, channel::Message, gateway::Ready, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};

use rand::seq::SliceRandom;

use crate::config::CONFIG;
use crate::ldap;
use crate::reaction_roles::{
    add_role_by_reaction, remove_role_by_reaction, sync_all_role_reactions,
};
use crate::user_management;
use crate::util::get_string_from_react;
use crate::voting;

pub struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if !(msg.content.starts_with(&CONFIG.command_prefix)) {
            if msg.content.contains(&format!("<@!{}>", CONFIG.bot_id)) // desktop mentions
                || msg.content.contains(&format!("<@{}>", CONFIG.bot_id))
            // mobile mentions
            {
                send_message!(
                    msg.channel_id,
                    &ctx.http,
                    MENTION_RESPONSES
                        .choose(&mut rand::thread_rng())
                        .expect("We couldn't get any sass")
                );
            }
            return;
        }
        let message_content: Vec<_> = msg.content[1..].splitn(2, ' ').collect();
        let content = if message_content.len() > 1 {
            message_content[1]
        } else {
            ""
        };
        match message_content[0] {
            "say" => println!("{:#?}", msg.content),
            "register" => user_management::Commands::register(ctx, msg.clone(), content),
            "verify" => user_management::Commands::verify(ctx, msg.clone(), content),
            "profile" => user_management::Commands::profile(ctx, msg.clone(), content),
            "set" => user_management::Commands::set_info(ctx, msg.clone(), content),
            "clear" => user_management::Commands::clear_info(ctx, msg.clone(), content),
            "move" => voting::Commands::move_something(ctx, msg.clone(), content),
            "motion" => voting::Commands::motion(ctx, msg.clone(), content),
            "poll" => voting::Commands::poll(ctx, msg.clone(), content),
            "cowsay" => voting::Commands::cowsay(ctx, msg.clone(), content),
            "source" => {
                let mut mesg = MessageBuilder::new();
                mesg.push(
                    "You want to look at my insides!? Eurgh.\nJust kidding, you can go over ",
                );
                mesg.push_italic("every inch");
                mesg.push(" of me here: https://gitlab.ucc.asn.au/UCC/discord-bot 😉");
                send_message!(msg.channel_id, &ctx.http, mesg.build());
            }
            "help" => {
                // Plaintext version, keep in case IRC users kick up a fuss
                // let mut message = MessageBuilder::new();
                // message.push_line(format!(
                //     "Use {}move <action> to make a circular motion",
                //     &CONFIG.command_prefix
                // ));
                // message.push_line(format!(
                //     "Use {}poll <proposal> to see what people think about something",
                //     &CONFIG.command_prefix
                // ));
                // send_message!(msg.channel_id, &ctx.http, message.build());

                let result = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|embed| {
                        embed.colour(serenity::utils::Colour::DARK_GREY);
                        embed.title("Commands for the UCC Bot");
                        embed.field("About", "This is UCC's own little in-house bot, please treat it nicely :)", false);
                        embed.field("Commitee", "`!move <text>` to make a circular motion\n\
                                                 `!poll <text>` to get people's opinions on something", false);
                        embed.field("Account", "`!register <ucc username>` to link your Discord and UCC account\n\
                                                `!profile <user>` to get the profile of a user\n\
                                                `!set <bio|git|web|photo>` to set that property of _your_ profile\n\
                                                `!updateroles` to update your registered roles", false);
                        embed.field("Fun", "`!cowsay <text>` to have a cow say your words\n\
                                            with no `<text>` it'll give you a fortune 😉", false);
                        embed
                    });
                    m
                });
                if let Err(why) = result {
                    error!("Error sending help embed: {:?}", why);
                }
            }
            // undocumented (in !help) functins
            "logreact" => {
                e!("Error deleting logreact prompt: {:?}", msg.delete(&ctx));
                send_message!(
                    msg.channel_id,
                    &ctx.http,
                    "React to this to log the ID (for the next 5min)"
                );
            }
            "ldap" => send_message!(
                msg.channel_id,
                &ctx.http,
                format!("{:?}", ldap::ldap_search(message_content[1]))
            ),
            "tla" => send_message!(
                msg.channel_id,
                &ctx.http,
                format!("{:?}", ldap::tla_search(message_content[1]))
            ),
            "updateroles" => user_management::Commands::update_registered_role(ctx, msg),
            _ => send_message!(
                msg.channel_id,
                &ctx.http,
                format!("Unrecognised command. Try {}help", &CONFIG.command_prefix)
            ),
        }
    }

    fn reaction_add(&self, ctx: Context, add_reaction: channel::Reaction) {
        match add_reaction.message(&ctx.http) {
            Ok(message) => match get_message_type(&message) {
                MessageType::RoleReactMessage if add_reaction.user_id.0 != CONFIG.bot_id => {
                    add_role_by_reaction(&ctx, message, add_reaction);
                    return;
                }
                _ if message.author.id.0 != CONFIG.bot_id
                    || add_reaction.user_id == CONFIG.bot_id =>
                {
                    return
                }
                MessageType::Motion => voting::reaction_add(ctx, add_reaction),
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
                    send_message!(message.channel_id, &ctx.http, msg.build());
                }
                _ => {}
            },
            Err(why) => error!("Failed to get react message {:?}", why),
        }
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: channel::Reaction) {
        match removed_reaction.message(&ctx.http) {
            Ok(message) => match get_message_type(&message) {
                MessageType::RoleReactMessage if removed_reaction.user_id != CONFIG.bot_id => {
                    remove_role_by_reaction(&ctx, message, removed_reaction);
                    return;
                }
                _ if message.author.id.0 != CONFIG.bot_id
                    || removed_reaction.user_id == CONFIG.bot_id =>
                {
                    return
                }
                MessageType::Motion => voting::reaction_remove(ctx, removed_reaction),
                _ => {}
            },
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
        sync_all_role_reactions(&ctx);
    }

    fn resume(&self, ctx: Context, _: serenity::model::event::ResumedEvent) {
        sync_all_role_reactions(&ctx);
    }
}

pub const MENTION_RESPONSES: &[&str] = &[
    "Oh hello there",
    "Stop bothering me. I'm busy.",
    "You know, I'm trying to keep track of this place. I don't need any more distractions.",
    "Don't you have better things to do?",
    "(sigh) what now?",
    "Yes, yes, I know I'm brilliant",
    "What do I need to do to catch a break around here? Eh.",
    "Mmmmhmmm. I'm still around, don't mind me.",
    "You know, some people would consider this rude. Luckily I'm not one of those people. In fact, I'm not even a person.",
    "Perhaps try bothering someone else for a change."
];

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
