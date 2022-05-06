use crate::cmd::{check_msg, defer_interaction, now_playing_embed, Res};
use serenity::async_trait;
use serenity::client::Context;
use serenity::http::Http;
use serenity::model::channel::ChannelType;
use serenity::model::gateway::Activity;
use serenity::model::id::ChannelId;
use serenity::model::interactions::application_command::{
    ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
};
use serenity::prelude::Mentionable;
use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};
use tokio::sync::Mutex;

struct TrackEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
    handler_lock: Arc<Mutex<Call>>,
    ctx: Arc<Mutex<Context>>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_track_list) = ctx {
            let handler = self.handler_lock.lock().await;
            if let Some(np) = handler.queue().current() {
                let metadata = np.metadata();

                let app_ctx = self.ctx.lock().await;
                app_ctx
                    .set_activity(Activity::listening(metadata.title.clone().unwrap()))
                    .await;

                check_msg(
                    self.chan_id
                        .send_message(&self.http, |m| {
                            now_playing_embed(m, metadata.clone());
                            m
                        })
                        .await,
                );
            } else {
                let app_ctx = self.ctx.lock().await;
                app_ctx.reset_presence().await;

                check_msg(
                    self.chan_id
                        .say(&self.http, "🕳 Queue is empty! That's sad... I guess...")
                        .await,
                );
            }
        }

        None
    }
}

struct ChannelDurationNotifier {
    chan_id: ChannelId,
    count: Arc<AtomicUsize>,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for ChannelDurationNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let count_before = self.count.fetch_add(1, Ordering::Relaxed);
        check_msg(
            self.chan_id
                .say(
                    &self.http,
                    &format!(
                        "I've been in this channel for {} minutes!",
                        count_before + 1
                    ),
                )
                .await,
        );

        None
    }
}

pub async fn join(ctx: &Context, cmd: &ApplicationCommandInteraction) -> Res {
    check_msg(
        cmd.create_interaction_response(&ctx.http, |response| {
            defer_interaction(response, None, true)
        })
        .await,
    );

    let guild = ctx.cache.guild(cmd.guild_id.unwrap()).await.unwrap();

    let channel_option = cmd
        .data
        .options
        .get(0)
        .expect("Expected user option")
        .resolved
        .as_ref()
        .unwrap();

    let channel = match channel_option {
        ApplicationCommandInteractionDataOptionValue::Channel(channel) => match channel.kind {
            ChannelType::Voice => ctx.cache.channel(channel.id).await.unwrap(),
            _ => {
                check_msg(
                    cmd.edit_original_interaction_response(&ctx.http, |response| {
                        response.content("Must be a voice channel")
                    })
                    .await,
                );
                return Ok(());
            }
        },
        _ => {
            check_msg(
                cmd.edit_original_interaction_response(&ctx.http, |response| {
                    response.content("Must provide a channel")
                })
                .await,
            );
            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (handle_lock, success) = manager.join(guild.id, channel.id()).await;

    if let Ok(_channel) = success {
        let chan_id = cmd.channel_id;
        let send_http = ctx.http.clone();
        let mut handle = handle_lock.lock().await;

        handle.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                chan_id,
                http: send_http,
                handler_lock: handle_lock.clone(),
                ctx: Arc::new(Mutex::new(ctx.clone())),
            },
        );

        // let send_http = ctx.http.clone();
        // handle.add_global_event(
        //     Event::Periodic(Duration::from_secs(60), None),
        //     ChannelDurationNotifier {
        //         chan_id,
        //         count: Default::default(),
        //         http: send_http,
        //     },
        // );

        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content(format!("Joined {}", channel.mention()))
            })
            .await,
        );
    } else {
        check_msg(
            cmd.edit_original_interaction_response(&ctx.http, |response| {
                response.content("Error joining the channel")
            })
            .await,
        );
    }

    Ok(())
}
