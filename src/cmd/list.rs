use serenity::{
    builder::CreateEmbed, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
};

use crate::cmd::{check_msg, defer_interaction, duration_format, Res};

pub async fn list(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
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
        let queue = handler.queue().current_queue();

        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content(format!(
                    "**{} track(s) in queue**\n{} first tracks:",
                    queue.len(),
                    10.min(queue.len())
                ));

                queue[..10.min(queue.len())].iter().for_each(|track| {
                    let meta = track.metadata().clone();
                    let mut e = CreateEmbed::default();
                    e.field(
                        "Title",
                        &meta
                            .title
                            .unwrap_or_else(|| "This shit has no title?".to_string()),
                        false,
                    );
                    if let Some(t) = meta.source_url {
                        e.field("URL", t, false);
                    }
                    e.field("Duration", duration_format(meta.duration), true);
                    if let Some(t) = meta.thumbnail {
                        e.thumbnail(t);
                    }

                    response.add_embed(e);
                });

                response
            })
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
