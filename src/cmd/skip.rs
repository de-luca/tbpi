use crate::cmd::{check_msg, defer_interaction, Res};
use serenity::builder::CreateEmbed;
use serenity::{
    builder::{CreateActionRow, CreateButton},
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
    check_msg(
        cmd.create_interaction_response(&ctx.http, |response| {
            defer_interaction(response, None, false)
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

        if meat_users.is_empty() || meat_users.iter().all(|k| yep_members.contains_key(k)) {
            let _ = queue.skip();
            check_msg(
                cmd.edit_original_interaction_response(&ctx.http, |response| {
                    response.content("⏭ Skipped current track")
                })
                .await,
            );
            return Ok(());
        }

        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response
                    .create_embed(|e| {
                        skip_embed(
                            e,
                            cmd.user.name.clone(),
                            yep_members.clone(),
                            nope_members.clone(),
                        )
                    })
                    .components(|c| c.add_action_row(SkipBtn::action_row()))
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
                        response
                            .kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|d| {
                                d.create_embed(|e| {
                                    skip_embed(
                                        e,
                                        cmd.user.name.clone(),
                                        yep_members.clone(),
                                        nope_members.clone(),
                                    )
                                })
                                .components(|c| c.add_action_row(SkipBtn::action_row()))
                            })
                    })
                    .await,
                );
            }
        }

        let content = match yep_members.len() > nope_members.len() {
            true => {
                let _ = queue.skip();
                "Vote to skip succeeded\n⏭ Skipped current track".to_string()
            }
            false => "Vote to skip failed.".to_string(),
        };

        cmd.edit_original_interaction_response(&ctx.http, |response| {
            response
                .components(|c| c.set_action_rows(Vec::new()))
                .create_embed(|e| {
                    skip_embed(
                        e,
                        cmd.user.name.clone(),
                        yep_members.clone(),
                        nope_members.clone(),
                    )
                    .description("Vote has ended.")
                })
        })
        .await
        .unwrap();

        check_msg(
            cmd.create_followup_message(&ctx.http, |r| r.content(&content))
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

fn skip_embed(
    embed: &mut CreateEmbed,
    initiator: String,
    yep_members: HashMap<UserId, String>,
    nope_members: HashMap<UserId, String>,
) -> &mut CreateEmbed {
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

    embed
        .title(format!(
            "{} wants to skip current track, who's with them?",
            initiator
        ))
        .description("You have 15 seconds to vote.")
        .field("Nope", nope, true)
        .field("Yep", yep, true)
}
