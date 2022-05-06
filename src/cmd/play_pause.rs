use crate::cmd::{check_msg, defer_interaction, Res};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::Mentionable,
};

pub enum Op {
    Pause,
    Resume,
}

pub async fn play_pause(ctx: &Context, cmd: &ApplicationCommandInteraction, op: Op) -> Res {
    check_msg(
        cmd.create_interaction_response(&ctx.http, |response| {
            defer_interaction(response, None, true)
        })
        .await,
    );

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
            cmd.edit_original_interaction_response(&ctx.http, |response| response.content(content))
                .await,
        );
    } else {
        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content("Not playing in a voice channel.")
            })
            .await,
        );
    }

    Ok(())
}
