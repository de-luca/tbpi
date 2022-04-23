use crate::cmd::{check_msg, interaction_reply, Res};
use serenity::{
    client::Context,
    model::gateway::Activity,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
};
use songbird::input::Restartable;

pub async fn queue(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
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
                cmd.create_interaction_response(&ctx.http, |response| {
                    interaction_reply(
                        response,
                        "Must provide a URL to a video or audio".to_string(),
                        true,
                    )
                })
                .await,
            );
            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                interaction_reply(response, "Must provide a valid URL".to_string(), true)
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
                    cmd.create_interaction_response(&ctx.http, |response| {
                        interaction_reply(response, "Error sourcing ffmpeg".to_string(), true)
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

        if handler.queue().len() == 1 {
            ctx.set_activity(Activity::listening(&title)).await;
        }

        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                interaction_reply(
                    response,
                    format!(
                        "Queued **{}** at position {}",
                        &title,
                        handler.queue().len(),
                    ),
                    true,
                )
            })
            .await,
        );
    } else {
        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                interaction_reply(
                    response,
                    "Not in a voice channel to play in".to_string(),
                    true,
                )
            })
            .await,
        );
    }

    Ok(())
}
