use serenity::{
    framework::standard::{
        macros::{command, group},
        *,
    },
    model::{channel, channel::Message},
    prelude::*,
    utils::MessageBuilder,
};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::config::CONFIG;
use crate::util::get_string_from_react;

#[command("move")]
#[min_args(1)]
#[usage = "<text>"]
/// Make a circular motion
fn circ_motion(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    info!("{} created a motion {}", msg.author.name, args.rest());
    if let Err(why) = msg.delete(ctx.clone()) {
        error!("Error deleting motion prompt: {:?}", why);
    }
    let result = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.author(|a| {
                a.name(&msg.author.name);
                a.icon_url(
                    msg.author
                        .static_avatar_url()
                        .expect("Expected author to have avatar"),
                );
                a
            });
            embed.colour(serenity::utils::Colour::GOLD);
            embed.title(format!("Motion to {}", args.rest()));
            let mut desc = MessageBuilder::new();
            desc.role(CONFIG.vote_role);
            desc.push(" take a look at this motion from ");
            desc.mention(&msg.author);
            embed.description(desc.build());
            embed.field("Status", "Under Consideration", true);
            embed.field("Votes", "For: 0\nAgainst: 0\nAbstain: 0", true);
            embed.timestamp(msg.timestamp.to_rfc3339());
            embed
        });
        m.reactions(vec![
            CONFIG.for_vote.to_string(),
            CONFIG.against_vote.to_string(),
            CONFIG.abstain_vote.to_string(),
            CONFIG.approve_react.to_string(),
            CONFIG.disapprove_react.to_string(),
        ]);
        m
    });
    result.map_err(|why| {
        error!("Error creating motion: {:?}", why);
        why
    })?;
    Ok(())
}

#[group]
#[commands(poll, circ_motion, motion)]
struct Voting;

#[command]
#[help_available(false)]
fn motion(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    send_message!(
        msg.channel_id,
        &ctx.http,
        "I hope you're not having a motion. You may have wanted to !move something instead."
    )?;
    Ok(())
}

#[command]
#[min_args(1)]
#[usage = "<text>"]
/// Get people's opinions on something
fn poll(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    info!("{} created a poll {}", msg.author.name, args.rest());
    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|embed| {
            embed.author(|a| {
                a.name(&msg.author.name);
                a.icon_url(
                    msg.author
                        .static_avatar_url()
                        .expect("Expected author to have avatar"),
                );
                a
            });
            embed.colour(serenity::utils::Colour::BLUE);
            embed.title(format!("Poll {}", args.rest()));
            let mut desc = MessageBuilder::new();
            desc.mention(&msg.author);
            desc.push(" wants to know what you think.");
            embed.description(desc.build());
            embed.timestamp(msg.timestamp.to_rfc3339());
            embed
        });
        m.reactions(vec![
            CONFIG.approve_react.to_string(),
            CONFIG.disapprove_react.to_string(),
            CONFIG.unsure_react.to_string(),
        ]);
        m
    }) {
        Err(why) => {
            error!("Error sending message: {:?}", why);
        }
        Ok(_) => {
            if let Err(why) = msg.delete(ctx) {
                error!("Error deleting motion prompt: {:?}", why);
            }
        }
    };
    Ok(())
}

#[derive(Debug, Clone)]
struct MotionInfo {
    votes: HashMap<String, Vec<serenity::model::user::User>>,
}

lazy_static! {
    static ref MOTIONS_CACHE: Mutex<HashMap<serenity::model::id::MessageId, MotionInfo>> =
        Mutex::new(HashMap::new());
}

fn get_cached_motion(ctx: &Context, msg: &Message) -> MotionInfo {
    let mut cached_motions = MOTIONS_CACHE.lock().unwrap();
    if !cached_motions.contains_key(&msg.id) {
        info!("Initialising representation of motion {:?}", msg.id);
        let this_motion = MotionInfo {
            votes: {
                let mut m = HashMap::new();
                m.insert(
                    CONFIG.for_vote.to_string(),
                    msg.reaction_users(ctx, CONFIG.for_vote.to_string(), Some(100), None)
                        .unwrap(),
                );
                m.insert(
                    CONFIG.against_vote.to_string(),
                    msg.reaction_users(ctx, CONFIG.against_vote.to_string(), Some(100), None)
                        .unwrap(),
                );
                m.insert(
                    CONFIG.abstain_vote.to_string(),
                    msg.reaction_users(ctx, CONFIG.abstain_vote.to_string(), Some(100), None)
                        .unwrap(),
                );
                m
            },
        };
        cached_motions.insert(msg.id, this_motion);
    }
    (*cached_motions.get(&msg.id).unwrap()).clone()
}
fn set_cached_motion(id: serenity::model::id::MessageId, motion_info: MotionInfo) {
    if let Some(motion) = MOTIONS_CACHE.lock().unwrap().get_mut(&id) {
        *motion = motion_info;
        return;
    }
    warn!("{}", "Couldn't find motion in cache to set");
}

macro_rules! tiebreaker {
    ($ctx: expr, $vote: expr, $motion_info: expr) => {
        if $motion_info.votes.get($vote).unwrap().iter().any(|u| {
            u.has_role($ctx, CONFIG.server_id, CONFIG.tiebreaker_role)
                .unwrap()
        }) {
            0.25
        } else {
            0.0
        }
    };
}

fn update_motion(
    ctx: &Context,
    msg: &mut Message,
    user: &serenity::model::user::User,
    change: &str,
    reaction: channel::Reaction,
) {
    let motion_info: MotionInfo = get_cached_motion(ctx, msg);

    let for_votes = motion_info.votes.get(&CONFIG.for_vote).unwrap().len() as isize - 1;
    let against_votes = motion_info.votes.get(&CONFIG.against_vote).unwrap().len() as isize - 1;
    let abstain_votes = motion_info.votes.get(&CONFIG.abstain_vote).unwrap().len() as isize - 1;

    let for_strength = for_votes as f32 + tiebreaker!(ctx, &CONFIG.for_vote, motion_info);
    let against_strength =
        against_votes as f32 + tiebreaker!(ctx, &CONFIG.against_vote, motion_info);
    let abstain_strength =
        abstain_votes as f32 + tiebreaker!(ctx, &CONFIG.abstain_vote, motion_info);

    let old_embed = msg.embeds[0].clone();
    let topic = old_embed.clone().title.unwrap();

    info!(
        "  {:10} {:6} {} on {}",
        user.name,
        change,
        get_string_from_react(&reaction.emoji),
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
            info!("Motion to {} now {}", topic, status);
            //
            let mut message = MessageBuilder::new();
            message.push_bold(topic);
            message.push(" is now ");
            message.push_bold(status);
            message.push_italic(format!(" (was {})", last_status));
            send_message!(CONFIG.announcement_channel, &ctx.http, message.build())
                .map(|_| ())
                .or(<Result<(), &dyn std::error::Error>>::Ok(()))
                .unwrap();
        }
    };

    if let Err(why) = msg.edit(ctx, |m| {
        m.embed(|e| {
            e.author(|a| {
                let old_author = old_embed.clone().author.expect("Expected author in embed");
                a.name(old_author.name);
                a.icon_url(
                    old_author
                        .icon_url
                        .expect("Expected embed author to have icon"),
                );
                a
            });
            e.title(&topic);
            e.description(old_embed.description.unwrap());
            let last_status_full = old_embed
                .fields
                .iter()
                .find(|f| f.name == "Status")
                .expect("No previous status")
                .clone()
                .value;
            if for_strength > (CONFIG.vote_pool_size as f32 / 2.0) {
                e.colour(serenity::utils::Colour::TEAL);
                update_status(e, "Passed", last_status_full, &topic);
            } else if against_strength + abstain_strength > (CONFIG.vote_pool_size as f32 / 2.0) {
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
                    CONFIG.vote_pool_size
                ),
                format!(
                    "For: {}\nAgainst: {}\nAbstain: {}",
                    for_votes, against_votes, abstain_votes
                ),
                true,
            );
            e.timestamp(
                old_embed
                    .timestamp
                    .expect("Expected embed to have timestamp"),
            );
            e
        })
    }) {
        error!("Error updating motion: {:?}", why);
    }
}

pub fn reaction_add(ctx: Context, add_reaction: channel::Reaction) {
    let react_as_string = get_string_from_react(&add_reaction.emoji);
    match add_reaction.message(&ctx.http) {
        Ok(mut message) => {
            guard!(let Ok(user) = add_reaction.user(&ctx) else {
                return
            });
            match user.has_role(&ctx, CONFIG.server_id, CONFIG.vote_role) {
                Ok(true) => {
                    // remove vote if already voted
                    for react in [
                        CONFIG.for_vote.to_string(),
                        CONFIG.against_vote.to_string(),
                        CONFIG.abstain_vote.to_string(),
                    ]
                    .iter()
                    .filter(|r| r != &&react_as_string)
                    {
                        for a_user in message
                            .reaction_users(&ctx, react.as_str(), None, None)
                            .unwrap()
                        {
                            if a_user.id.0 == user.id.0 {
                                if let Err(why) = add_reaction.delete(&ctx) {
                                    error!("Error deleting react: {:?}", why);
                                };
                                return;
                            }
                        }
                    }
                    // remove 'illegal' reacts
                    if !CONFIG.allowed_reacts().contains(&react_as_string) {
                        if let Err(why) = add_reaction.delete(&ctx) {
                            error!("Error deleting react: {:?}", why);
                        };
                        return;
                    }
                    // update motion
                    let mut motion_info = get_cached_motion(&ctx, &message);
                    if let Some(vote) = motion_info.votes.get_mut(&react_as_string) {
                        vote.retain(|u| u.id != user.id);
                        vote.push(user.clone());
                    }
                    set_cached_motion(message.id, motion_info);
                    update_motion(&ctx, &mut message, &user, "add", add_reaction);
                }
                Ok(false) => {
                    if ![
                        CONFIG.approve_react.to_string(),
                        CONFIG.disapprove_react.to_string(),
                    ]
                    .contains(&react_as_string)
                    {
                        if let Err(why) = add_reaction.delete(&ctx) {
                            error!("Error deleting react: {:?}", why);
                        };
                        return;
                    }
                }
                Err(why) => {
                    error!("Error getting user role: {:?}", why);
                }
            }
        }
        Err(why) => {
            error!("Error processing react: {:?}", why);
        }
    }
}

pub fn reaction_remove(ctx: Context, removed_reaction: channel::Reaction) {
    match removed_reaction.message(&ctx.http) {
        Ok(mut message) => {
            if let Ok(user) = removed_reaction.user(&ctx) {
                let mut motion_info = get_cached_motion(&ctx, &message);
                if let Some(vote) = motion_info
                    .votes
                    .get_mut(&get_string_from_react(&removed_reaction.emoji))
                {
                    vote.retain(|u| u.id != user.id);
                }
                set_cached_motion(message.id, motion_info);
                update_motion(&ctx, &mut message, &user, "remove", removed_reaction);
            }
        }
        Err(why) => {
            error!("Error getting user role: {:?}", why);
        }
    }
}
