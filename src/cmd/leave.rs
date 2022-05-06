use crate::cmd::{check_msg, defer_interaction, Res};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn leave(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
    check_msg(
        cmd.create_interaction_response(&ctx.http, |response| {
            defer_interaction(response, None, true)
        })
        .await,
    );

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
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content("Left voice channel")
            })
            .await,
        );
    } else {
        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content("Not in a voice channel")
            })
            .await,
        );
    }

    Ok(())
}
