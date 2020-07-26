use serenity::framework::standard::{macros::*, *};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[group]
#[commands(cowsay)]
struct Fun;

#[command]
fn cowsay(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let content = args.rest();
    let output = if !content.trim().is_empty() {
        let mut text = content.to_owned();
        text.escape_default();
        // Guess what buddy! You definitely are passing a string to cowsay
        text.insert(0, '\'');
        text.insert(text.len(), '\'');
        std::process::Command::new("cowsay")
            .arg(text)
            .output()
            .map_err(|e| {
                error!("Failed to execute cowsay: {:?}", e);
                e
            })?
    } else {
        static OPTIONS: &[&str] = &[
            "www",
            "hellokitty",
            "bud-frogs",
            "koala",
            "suse",
            "three-eyes",
            "pony-smaller",
            "sheep",
            "vader",
            "cower",
            "moofasa",
            "elephant",
            "flaming-sheep",
            "skeleton",
            "snowman",
            "tux",
            "apt",
            "moose",
        ];
        let o = std::process::Command::new("fortune")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                error!("Failed to execute fortune: {:?}", e);
                e
            })?;
        use rand::seq::SliceRandom;
        std::process::Command::new("cowsay")
            .stdin(o.stdout.unwrap())
            .arg("-f")
            .arg(format!(
                "/usr/share/cowsay/cows/{}.cow",
                OPTIONS
                    .choose(&mut rand::thread_rng())
                    .unwrap_or(&"default")
            ))
            .output()
            .map_err(|e| {
                error!("Failed to execute cowsay: {:?}", e);
                e
            })?
    };
    let mut message = MessageBuilder::new();
    message.push_codeblock_safe(
        String::from_utf8(output.stdout).expect("unable to parse stdout to String"),
        None,
    );
    send_message!(msg.channel_id, &ctx.http, message.build())?;
    Ok(())
}
