use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
    model::interactions::{
        InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
    },
};

use crate::cmd::{check_msg, duration_format, interaction_reply, Res};

pub async fn list(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(cmd.guild_id.unwrap()) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue().current_queue();

        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                            .content(format!(
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

                            message.add_embed(e);
                        });

                        message
                    })
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
