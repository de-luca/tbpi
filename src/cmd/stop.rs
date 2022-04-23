use crate::cmd::{check_msg, interaction_reply, Res};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateComponents},
    client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
    model::interactions::message_component::ButtonStyle,
    model::interactions::{
        InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
    },
    prelude::Mentionable,
};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::Duration;

enum StopBtn {
    Cancel,
    Proceed,
}

impl Display for StopBtn {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Cancel => write!(f, "Cancel"),
            Self::Proceed => write!(f, "Proceed"),
        }
    }
}

impl StopBtn {
    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self.to_string().to_ascii_lowercase());
        b.label(self);
        match self {
            StopBtn::Cancel => b.style(ButtonStyle::Danger),
            StopBtn::Proceed => b.style(ButtonStyle::Primary),
        };
        b
    }

    fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_button(StopBtn::Cancel.button());
        ar.add_button(StopBtn::Proceed.button());
        ar
    }
}

pub async fn stop(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(cmd.guild_id.unwrap()) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource);
                response.interaction_response_data(|d| {
                    d.content("You are about to stop playback and purge the queue")
                        .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                        .components(|c| c.add_action_row(StopBtn::action_row()))
                })
            })
            .await,
        );

        let message = cmd.get_interaction_response(&ctx.http).await?;
        let res = message
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(30))
            .await;

        match res {
            None => check_msg(
                cmd.edit_original_interaction_response(&ctx.http, |response| {
                    response
                        .content("Canceled")
                        .components(|c| c.set_action_rows(Vec::new()))
                })
                .await,
            ),
            Some(res) => {
                match res.data.custom_id.as_str() {
                    "proceed" => {
                        let _ = queue.stop();
                        res.delete_original_interaction_response(&ctx.http)
                            .await
                            .unwrap();
                        check_msg(
                            res.create_interaction_response(&ctx.http, |response| {
                                response.interaction_response_data(|d| {
                                    d.content(format!(
                                        "⏹ {} stopped and cleared queue",
                                        cmd.user.mention()
                                    ))
                                })
                            })
                            .await,
                        );
                    }
                    _ => {
                        check_msg(
                            res.create_interaction_response(&ctx.http, |response| {
                                response.kind(InteractionResponseType::UpdateMessage);
                                response.interaction_response_data(|d| {
                                    d.content("Canceled")
                                        .set_components(CreateComponents::default())
                                })
                            })
                            .await,
                        );
                    }
                };
            }
        }
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
