use serenity::framework::standard::{macros::*, *};
use serenity::{model::channel::Message, prelude::*, utils::*};

#[group]
#[commands(source)]
struct General;

#[command]
/// Get the source code of the bot
fn source(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let mut mesg = MessageBuilder::new();
    mesg.push("You want to look at my insides!? Eurgh.\nJust kidding, you can go over ");
    mesg.push_italic("every inch");
    mesg.push(" of me here: https://github.com/uwapcs/discord-bot 😉");
    send_message!(msg.channel_id, &ctx.http, mesg.build())?;
    Ok(())
}

// Repeats what the user passed as argument but ensures that user and role
// mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
#[command]
/// Repeat things back but drop user and role mentions
fn say(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings);

    if let Err(why) = msg.channel_id.say(&ctx.http, &content) {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}
