use crate::cmd::{check_msg, interaction_reply, Res};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::Mentionable,
};

pub enum Op {
    Pause,
    Resume,
}

pub async fn play_pause(ctx: &Context, cmd: &ApplicationCommandInteraction, op: Op) -> Res {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(cmd.guild_id.unwrap()) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let content = match op {
            Op::Pause => {
                let _ = queue.pause();
                format!("⏸ {} paused current track", cmd.user.mention())
            }
            Op::Resume => {
                let _ = queue.resume();
                format!("▶️ {} resumed current track", cmd.user.mention())
            }
        };

        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                interaction_reply(response, content, false)
            })
            .await,
        );
    } else {
        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                interaction_reply(
                    response,
                    "Not playing in a voice channel.".to_string(),
                    true,
                )
            })
            .await,
        );
    }

    Ok(())
}
