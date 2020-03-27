use rand::seq::SliceRandom;
use regex::Regex;
use serenity::{
    model::{channel::Message, guild::Member},
    prelude::*,
    utils::MessageBuilder,
};
use std::process::{Command, Stdio};
use url::Url;

use crate::config::CONFIG;
use crate::database;
use crate::ldap::ldap_exists;
use crate::token_management::*;

pub fn new_member(ctx: &Context, mut new_member: Member) {
    let mut message = MessageBuilder::new();
    message.push("Nice to see you here ");
    message.mention(&new_member);
    message.push_line("! Would you care to introduce yourself?");
    message.push_line("If you're not sure where to start, perhaps you could tell us about your projects, your first computer…");
    message.push_line("You should also know that we follow the Freenode Channel Guidelines: https://freenode.net/changuide, and try to avoid defamatory content.");
    message.push_line("Make sure to check out ");
    message.mention(&CONFIG.readme_channel);
    message.push_line(" to get yourself some roles for directed pings 😊, and ");
    message.push_mono(format!("{}register username", CONFIG.command_prefix));
    message.push_line(" to link to your UCC account.");
    send_message!(CONFIG.welcome_channel, &ctx, message.build());

    let mut message = MessageBuilder::new();
    message.push(format!("Say hi to {} in ", new_member.display_name()));
    message.mention(&CONFIG.welcome_channel);
    send_message!(CONFIG.main_channel, &ctx, message.build());

    if let Err(why) = new_member.add_role(&ctx.http, CONFIG.unregistered_member_role) {
        error!("Error adding user role: {:?}", why);
    };
}

fn member_nickname(member: &database::Member) -> String {
    let username = member.username.clone();
    if let Some(tla) = member.tla.clone() {
        if username.to_uppercase() == tla {
            return format!("{}", username);
        } else {
            return format!("{} [{}]", username, tla);
        }
    } else {
        return format!("{}", username);
    }
}

pub const RANDOM_SASS: &[&str] = &[
    "Please. As if I'd fall for that.",
    "Did you really think a stunt like that would work?",
    "Nothing slips past me.",
    "Did you even read the first line of !help?",
    "I never treated you this badly.",
];

pub const RESERVED_NAMES: &[&str] = &[
    "committee",
    "committee-only",
    "ucc",
    "ucc-announce",
    "tech",
    "wheel",
    "door",
    "coke",
];

pub struct Commands;
impl Commands {
    pub fn register(ctx: Context, msg: Message, account_name: &str) {
        if account_name.is_empty() {
            send_message!(
                msg.channel_id,
                &ctx.http,
                format!("Usage: {}register <username>", CONFIG.command_prefix)
            );
            return;
        }
        if RESERVED_NAMES.contains(&account_name) || database::username_exists(account_name) {
            send_message!(
                msg.channel_id,
                &ctx.http,
                RANDOM_SASS
                    .choose(&mut rand::thread_rng())
                    .expect("We couldn't get any sass")
            );
            return;
        }
        if !ldap_exists(account_name) {
            send_message!(
                msg.channel_id,
                &ctx.http,
                format!(
                    "I couldn't find an account with the username '{}'",
                    account_name
                )
            );
            return;
        }
        send_message!(
            msg.channel_id,
            &ctx.http,
            format!(
                "Ok {}, see the email I've just sent you to complete the link",
                account_name
            )
        );

        e!("Error deleting register message: {:?}", msg.delete(ctx));

        let message = Command::new("echo").arg(format!("<h3>Link your Discord account</h3>\
                                                        <p>Hi {}, to complete the link, go to the discord server and enter\
                                                        <pre>{}verify {}</pre>\
                                                        </p><sub>The UCC discord bot</sub>",
                                                        account_name, CONFIG.command_prefix, generate_token(&msg.author, account_name))).stdout(Stdio::piped()).spawn().expect("Unable to spawn echo command");
        match Command::new("mutt")
            .arg("-e")
            .arg("set content_type=text/html")
            .arg("-e")
            .arg("set realname=\"UCC Discord Bot\"")
            .arg("-s")
            .arg("Discord account link token")
            .arg(format!("{}@ucc.asn.au", account_name))
            .stdin(message.stdout.unwrap())
            .output()
        {
            Ok(_) => info!("Email sent to {}", account_name),
            Err(why) => error!("Unable to send message with mutt {:?}", why),
        };
    }
    pub fn verify(ctx: Context, msg: Message, token: &str) {
        match parse_token(&msg.author, token) {
            Ok(name) => {
                e!(
                    "Unable to get member: {:?}",
                    serenity::model::id::GuildId(CONFIG.server_id)
                        .member(ctx.http.clone(), msg.author.id)
                        .map(|mut member| {
                            let full_member = database::add_member(&msg.author.id.0, &name);
                            e!(
                                "Unable to remove role: {:?}",
                                member.remove_role(&ctx.http, CONFIG.unregistered_member_role)
                            );
                            e!(
                                "Unable to add role: {:?}",
                                member.add_role(&ctx.http, CONFIG.registered_member_role)
                            );
                            e!(
                                "Unable to edit nickname: {:?}",
                                member.edit(&ctx.http, |m| {
                                    m.nickname(member_nickname(&full_member));
                                    m
                                })
                            );
                            let mut verification_message = MessageBuilder::new();
                            verification_message.push(format!("Verification was sucessful {}. To proide a friendly introduction to yourself consider doing ", &full_member.username));
                            verification_message.push_mono(format!("{}set bio <info>", CONFIG.command_prefix));
                            send_message!(
                                msg.channel_id,
                                ctx.http.clone(),
                                verification_message.build()
                            );
                        })
                );
            }
            Err(reason) => send_message!(
                msg.channel_id,
                &ctx.http,
                format!("Verification error: {:?}", reason)
            ),
        }
        e!("Error deleting register message: {:?}", msg.delete(&ctx));
    }
    pub fn profile(ctx: Context, msg: Message, name: &str) {
        let possible_member: Option<database::Member> = match if name.trim().is_empty() {
            database::get_member_info(&msg.author.id.0)
        } else {
            database::get_member_info_from_username(&name)
        } {
            Ok(member) => Some(member),
            Err(why) => {
                warn!("Could not find member {:?}", why);
                if name.len() != 3 {
                    None
                } else {
                    match database::get_member_info_from_tla(&name.to_uppercase()) {
                        Ok(member) => Some(member),
                        Err(_) => None,
                    }
                }
            }
        };
        if possible_member.is_none() {
            send_message!(
                msg.channel_id,
                &ctx.http,
                "Sorry, I couldn't find that profile (you need to !register for a profile)"
            );
            return;
        }
        let member = possible_member.unwrap();
        let result = msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|embed| {
                embed.colour(serenity::utils::Colour::LIGHTER_GREY);
                embed.footer(|f| {
                    let user = &ctx
                        .http
                        .get_user(member.discord_id.clone() as u64)
                        .expect("We expected this user to exist... they didn't ;(");
                    f.text(&user.name);
                    f.icon_url(
                        user.static_avatar_url()
                            .expect("Expected user to have avatar"),
                    );
                    f
                });
                if let Some(name) = member.name.clone() {
                    embed.title(name);
                }
                if let Some(photo) = member.photo.clone() {
                    embed.thumbnail(photo);
                }
                embed.field("Username", &member.username, true);
                if let Some(tla) = member.tla.clone() {
                    embed.field("TLA", tla, true);
                }
                if let Some(bio) = member.biography.clone() {
                    embed.field("Bio", bio, false);
                }
                if let Some(study) = member.study.clone() {
                    embed.field("Area of study", study, false);
                }
                if let Some(git) = member.github.clone() {
                    embed.field("Git", git, false);
                }
                if let Some(web) = member.website.clone() {
                    embed.field("Website", web, false);
                }
                embed
            });
            m
        });
        if let Err(why) = result {
            error!("Error sending profile embed: {:?}", why);
        }
    }
    pub fn set_info(ctx: Context, msg: Message, info: &str) {
        if info.trim().is_empty() {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|embed| {
                        embed.colour(serenity::utils::Colour::LIGHT_GREY);
                        embed.title("Usage");
                        embed.description(
                            format!(
                                "`{}set <field> <info>` or `{}clear <field>`",
                                CONFIG.command_prefix,
                                CONFIG.command_prefix,
                            )
                        );
                        embed.field("Biography", format!("`{}set bio <info>`\nBe friendly! Provide a little introduction to yourself.", CONFIG.command_prefix), false);
                        embed.field("Git", format!("`{}set git <url>`\nA link to your git forge profile. Also takes a github username for convinience", CONFIG.command_prefix), false);
                        embed.field("Photo", format!("`{}set photo <url>`\nPut a face to a name! Provide a profile photo.", CONFIG.command_prefix), false);
                        embed.field("Website", format!("`{}set web <info>`\nGot a personal website? Share it here :)", CONFIG.command_prefix), false);
                        embed.field("Studying", format!("`{}set study <info>`\nYou're (probably) a Uni student, what's your major?", CONFIG.command_prefix), false);
                        embed
                    });
                    m
                })
                .expect("Failed to send usage help embed");
            return;
        }
        let info_content: Vec<_> = info.splitn(2, ' ').collect();
        let mut property = String::from(info_content[0]);
        property = property.replace("github", "git");
        if info_content.len() == 1
            || !vec!["bio", "git", "web", "photo", "study"].contains(&property.as_str())
        {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|embed| {
                        embed.colour(serenity::utils::Colour::LIGHT_GREY);
                        embed.title("Usage");
                        embed.field(
                            match property.as_str() {
                                "bio" => "Biography",
                                "git" => "Git Forge Profile",
                                "photo" => "Profile Photo",
                                "web" => "Personal Website",
                                "study" => "Area of study",
                                _ => "???",
                            },
                            format!(
                                "`{}set {} <info>` or `{}clear {}`\n{}",
                                CONFIG.command_prefix,
                                property,
                                CONFIG.command_prefix,
                                property,
                                match property.as_str() {
                                    "bio" => "Some information about yourself :)",
                                    "git" => "A url to your git{hub,lab} account",
                                    "photo" => "A url to a profile photo online",
                                    "web" => "A url to your website/webpage",
                                    "study" => "Your degree title",
                                    _ => "Whatever you want, because this does absolutely nothing.",
                                }
                            ),
                            false,
                        );
                        embed
                    });
                    m
                })
                .expect("Failed to send usage embed");
            return;
        }
        let mut value = info_content[1].to_string();

        if vec!["git", "photo", "web"].contains(&property.as_str()) {
            if Url::parse(&value).is_err() {
                let user_regex = Regex::new(r"^\w+$").unwrap();
                if property == "git" && user_regex.is_match(&value) {
                    value = format!("github.com/{}", value);
                }
                value = format!("https://{}", value);
                if Url::parse(&value).is_err() {
                    send_message!(
                        msg.channel_id,
                        &ctx.http,
                        "That ain't a URL where I come from..."
                    );
                    return;
                }
            }
        }
        guard!(let Ok(member) = database::get_member_info(&msg.author.id.0) else {
            send_message!(
                msg.channel_id,
                &ctx.http,
                format!(
                    "You don't seem to have a profile. {}register to get one",
                    CONFIG.command_prefix
                )
            );
            return
        });
        let set_property = match property.as_str() {
            "bio" => database::set_member_bio(&msg.author.id.0, Some(&value)),
            "git" => database::set_member_git(&msg.author.id.0, Some(&value)),
            "photo" => database::set_member_photo(&msg.author.id.0, Some(&value)),
            "web" => database::set_member_website(&msg.author.id.0, Some(&value)),
            "study" => database::set_member_study(&msg.author.id.0, Some(&value)),
            _ => Err(diesel::result::Error::NotFound),
        };
        match set_property {
            Ok(_) => {
                if property == "git" && member.photo == None {
                    let git_url = Url::parse(&value).unwrap(); // we parsed this earlier and it was fine
                    match git_url.host_str() {
                        Some("github.com") => {
                            if let Some(mut path_segments) = git_url.path_segments() {
                                database::set_member_photo(
                                    &msg.author.id.0,
                                    Some(
                                        format!(
                                            "https://github.com/{}.png",
                                            path_segments.next().expect("URL doesn't have a path")
                                        )
                                        .as_str(),
                                    ),
                                )
                                .expect("Attempt to set member photo failed");
                            } else {
                                info!("Git path added (2), {}", git_url.path());
                            }
                        }
                        _ => info!("Git path added, {}", git_url.path()),
                    }
                }
            }
            Err(why) => {
                error!(
                    "Umable to set property {} to {} in DB {:?}",
                    property, value, why
                );
                send_message!(msg.channel_id, &ctx.http, "Failed to set property. Ooops.");
            }
        }
        if let Err(why) = msg.delete(&ctx) {
            error!("Error deleting set profile property: {:?}", why);
        }
    }
    pub fn clear_info(ctx: Context, msg: Message, field: &str) {
        if field.trim().is_empty() {
            // just show the help page from set_info
            Commands::set_info(ctx, msg, "");
            return;
        }
        let clear_property = match field {
            "bio" => database::set_member_bio(&msg.author.id.0, None),
            "git" => database::set_member_git(&msg.author.id.0, None),
            "photo" => database::set_member_photo(&msg.author.id.0, None),
            "web" => database::set_member_website(&msg.author.id.0, None),
            "study" => database::set_member_study(&msg.author.id.0, None),
            _ => Err(diesel::result::Error::NotFound),
        };
    }
}
