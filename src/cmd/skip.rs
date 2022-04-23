use crate::cmd::{check_msg, interaction_reply, Res};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateInteractionResponse},
    client::Context,
    futures::StreamExt,
    model::id::UserId,
    model::interactions::application_command::ApplicationCommandInteraction,
    model::interactions::message_component::ButtonStyle,
    model::interactions::InteractionResponseType,
};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::Duration;

enum SkipBtn {
    Yep,
    Nope,
}

impl Display for SkipBtn {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Yep => write!(f, "Yep"),
            Self::Nope => write!(f, "Nope"),
        }
    }
}

impl SkipBtn {
    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self.to_string().to_ascii_lowercase());
        b.label(self);
        match self {
            SkipBtn::Nope => b.style(ButtonStyle::Danger),
            SkipBtn::Yep => b.style(ButtonStyle::Primary),
        };
        b
    }

    fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_button(SkipBtn::Nope.button());
        ar.add_button(SkipBtn::Yep.button());
        ar
    }
}

pub async fn skip(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(cmd.guild_id.unwrap()) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let users = ctx
            .cache
            .guild_channel(handler.current_channel().unwrap().0)
            .await
            .unwrap()
            .members(&ctx.cache)
            .await?;

        let meat_users = users
            .iter()
            .filter(|u| !u.user.bot)
            .map(|u| u.user.id)
            .collect::<Vec<UserId>>();

        let mut nope_members: HashMap<UserId, String> = HashMap::new();
        let mut yep_members: HashMap<UserId, String> = HashMap::new();
        yep_members.insert(cmd.user.id, cmd.user.name.clone());

        if meat_users.is_empty() || yep_members.keys().cloned().all(|k| meat_users.contains(&k)) {
            let _ = queue.skip();
            check_msg(
                cmd.create_interaction_response(&ctx.http, |response| {
                    interaction_reply(response, "⏭ Skipped current track".to_string(), true)
                })
                .await,
            );

            return Ok(());
        }

        check_msg(
            cmd.create_interaction_response(&ctx.http, |response| {
                skip_embed(
                    response,
                    InteractionResponseType::ChannelMessageWithSource,
                    cmd.user.name.clone(),
                    yep_members.clone(),
                    nope_members.clone(),
                )
            })
            .await,
        );

        let message = cmd.get_interaction_response(&ctx.http).await?;
        let mut res = message
            .await_component_interactions(&ctx)
            .timeout(Duration::from_secs(15))
            .await;

        while let Some(vote) = res.next().await {
            if meat_users.contains(&vote.user.id) {
                match vote.data.custom_id.as_str() {
                    "yep" => {
                        let _ = &yep_members.insert(vote.user.id, vote.user.name.clone());
                        let _ = &nope_members.remove(&vote.user.id);
                    }
                    _ => {
                        let _ = &nope_members.insert(vote.user.id, vote.user.name.clone());
                        let _ = &yep_members.remove(&vote.user.id);
                    }
                };

                check_msg(
                    vote.create_interaction_response(&ctx.http, |response| {
                        skip_embed(
                            response,
                            InteractionResponseType::UpdateMessage,
                            cmd.user.name.clone(),
                            yep_members.clone(),
                            nope_members.clone(),
                        )
                    })
                    .await,
                );
            }
        }

        let content = match yep_members.len() > nope_members.len() {
            true => {
                let _ = queue.skip();
                format!(
                    "Vote to skip succeeded ({}/{})",
                    yep_members.len(),
                    nope_members.len()
                )
            }
            false => format!(
                "Vote to skip failed ({}/{})",
                nope_members.len(),
                yep_members.len()
            ),
        };

        check_msg(
            cmd.create_followup_message(&ctx.http, |r| r.content(&content))
                .await,
        );
        cmd.delete_original_interaction_response(&ctx.http)
            .await
            .unwrap();
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

fn skip_embed(
    response: &mut CreateInteractionResponse,
    kind: InteractionResponseType,
    initiator: String,
    yep_members: HashMap<UserId, String>,
    nope_members: HashMap<UserId, String>,
) -> &mut CreateInteractionResponse {
    let yep = match yep_members.is_empty() {
        true => "-".to_string(),
        false => yep_members
            .values()
            .cloned()
            .collect::<Vec<String>>()
            .join("\n"),
    };
    let nope = match nope_members.is_empty() {
        true => "-".to_string(),
        false => nope_members
            .values()
            .cloned()
            .collect::<Vec<String>>()
            .join("\n"),
    };

    response.kind(kind).interaction_response_data(|d| {
        d.create_embed(|e| {
            e.title(format!(
                "{} wants to skip current track, who's with them?",
                initiator
            ))
            .description("You have 15 seconds to vote.")
            .field("Nope", nope, true)
            .field("Yep", yep, true)
        })
        .components(|c| c.add_action_row(SkipBtn::action_row()))
    })
}
