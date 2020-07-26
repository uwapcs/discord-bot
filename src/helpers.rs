use serenity::{client, framework::standard, model};

#[standard::macros::help]
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 아녕!\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
// Used to indicate sub-groups (if any)
#[indention_prefix = "+"]
// If a user lacks permissions for a command, hide the command
#[lacking_permissions = "Hide"]
// If the user is lacking the role, it is displayed
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
fn help(
    context: &mut client::Context,
    msg: &model::channel::Message,
    args: standard::Args,
    help_options: &'static standard::HelpOptions,
    groups: &[&'static standard::CommandGroup],
    owners: std::collections::HashSet<model::id::UserId>,
) -> standard::CommandResult {
    standard::help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

pub fn sassy(context: &mut client::Context, msg: &model::channel::Message) {
    use rand::seq::SliceRandom;
    send_message!(
        msg.channel_id,
        &context.http,
        if let Some(ref response) = crate::config::CONFIG.mention_responses {
            response
                .choose(&mut rand::thread_rng())
                .expect("We couldn't get any sass")
        } else {
            return;
        }
    )
    .map(|_| ())
    .or(<Result<(), &dyn std::error::Error>>::Ok(()))
    .unwrap();
}

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Motion,
    Role,
    RoleReactMessage,
    Poll,
    Misc,
}

pub fn get_message_type(message: &model::channel::Message) -> MessageType {
    if crate::config::CONFIG
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
