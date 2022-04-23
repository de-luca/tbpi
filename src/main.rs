mod cmd;

use crate::{
    cmd::join::join,
    cmd::leave::leave,
    cmd::list::list,
    cmd::play_pause::{play_pause, Op},
    cmd::queue::queue,
    cmd::skip::skip,
    cmd::stop::stop,
    cmd::{check_msg, interaction_reply},
};
use serenity::{
    async_trait,
    client::{bridge::gateway::GatewayIntents, Client, Context, EventHandler},
    model::{
        channel::ChannelType,
        gateway::Ready,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType},
            Interaction,
        },
    },
};
use songbird::SerenityInit;
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        ctx.reset_presence().await;

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("join")
                        .description("Request bot to join a voice channel")
                        .create_option(|option| {
                            option
                                .name("channel")
                                .description("The voice channel to join")
                                .kind(ApplicationCommandOptionType::Channel)
                                .channel_types(&[ChannelType::Voice])
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("leave")
                        .description("Request bot to leave its current voice channel")
                })
                .create_application_command(|command| {
                    command
                        .name("queue")
                        .description("Queue track")
                        .create_option(|option| {
                            option
                                .name("url")
                                .description("The YouTube url to queue")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command.name("pause").description("Pause current track")
                })
                .create_application_command(|command| {
                    command.name("resume").description("Resume current track")
                })
                .create_application_command(|command| {
                    command.name("skip").description("Skip current track")
                })
                .create_application_command(|command| {
                    command
                        .name("stop")
                        .description("Stop and purge tracks queue")
                })
                .create_application_command(|command| {
                    command.name("list").description("List queue content")
                })
        })
        .await;

        commands.unwrap().iter().for_each(|c| {
            println!("Created GLOBAL [{}] /{}", c.id, c.name);
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let _ = match command.data.name.as_str() {
                "join" => join(&ctx, &command).await,
                "leave" => leave(&ctx, &command).await,
                "queue" => queue(&ctx, &command).await,
                "pause" => play_pause(&ctx, &command, Op::Pause).await,
                "resume" => play_pause(&ctx, &command, Op::Resume).await,
                "skip" => skip(&ctx, &command).await,
                "stop" => stop(&ctx, &command).await,
                "list" => list(&ctx, &command).await,

                _ => {
                    return check_msg(
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                interaction_reply(response, "not implemented :(".to_string(), false)
                            })
                            .await,
                    )
                }
            };
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .intents(GatewayIntents::all())
        .application_id(application_id)
        .register_songbird()
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
