use crate::cmd::{check_msg, interaction_reply, Res};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn leave(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
    let guild_id = cmd.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                cmd.channel_id
                    .say(&ctx.http, &format!("Failed: {:?}", e))
                    .await,
            );
        }

        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                interaction_reply(response, "Left voice channel".to_string(), true)
            })
            .await,
        );
    } else {
        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                interaction_reply(response, "Not in a voice channel".to_string(), true)
            })
            .await,
        );
    }

    Ok(())
}
