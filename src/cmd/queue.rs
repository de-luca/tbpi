use crate::cmd::{check_msg, defer_interaction, Res};
use serenity::{
    client::Context,
    model::gateway::Activity,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
};
use songbird::input::Restartable;

pub async fn queue(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
    check_msg(
        cmd.create_interaction_response(&ctx.http, |response| {
            defer_interaction(response, None, true)
        })
        .await,
    );

    let url_option = cmd
        .data
        .options
        .get(0)
        .expect("Expected user option")
        .resolved
        .as_ref()
        .unwrap();

    let url = match url_option {
        ApplicationCommandInteractionDataOptionValue::String(url) => url.clone(),
        _ => {
            check_msg(
                cmd.edit_original_interaction_response(&ctx.http, |response| {
                    response.content("Must provide a URL to a video or audio")
                })
                .await,
            );
            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content("Must provide a valid URL")
            })
            .await,
        );
        return Ok(());
    }

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let guild_id = cmd.guild_id.unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);
                check_msg(
                    cmd.edit_original_interaction_response(&ctx.http, |response| {
                        response.content("Error sourcing ffmpeg")
                    })
                    .await,
                );
                return Ok(());
            }
        };

        handler.enqueue_source(source.into());

        let title = handler
            .queue()
            .current_queue()
            .last()
            .unwrap()
            .metadata()
            .title
            .clone()
            .unwrap();

        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content(format!(
                    "Queued **{}** at position {}",
                    &title,
                    handler.queue().len()
                ))
            })
            .await,
        );

        if handler.queue().len() == 1 {
            ctx.set_activity(Activity::listening(&title)).await;
        }
    } else {
        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content("Not in a voice channel to play in")
            })
            .await,
        );
    }

    Ok(())
}
