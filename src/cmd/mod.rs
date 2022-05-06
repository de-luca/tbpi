use serenity::builder::{CreateInteractionResponse, CreateMessage};
use serenity::model::interactions::{
    InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
};
use serenity::Result as SerenityResult;
use songbird::input::Metadata;
use std::{error::Error, time::Duration};

pub mod join;
pub mod leave;
pub mod list;
pub mod play_pause;
pub mod queue;
pub mod skip;
pub mod stop;

pub type Res = Result<(), Box<dyn Error>>;

pub fn check_msg<T>(result: SerenityResult<T>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

pub fn defer_interaction(
    response: &mut CreateInteractionResponse,
    content: Option<String>,
    ephemeral: bool,
) -> &mut CreateInteractionResponse {
    response
        .kind(InteractionResponseType::DeferredChannelMessageWithSource)
        .interaction_response_data(|message| {
            if let Some(c) = content {
                message.content(c);
            }
            if ephemeral {
                message.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
            }
            message
        })
}

pub fn interaction_reply(
    response: &mut CreateInteractionResponse,
    content: String,
    ephemeral: bool,
) -> &mut CreateInteractionResponse {
    response
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|message| {
            message.content(content);
            if ephemeral {
                message.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
            }
            message
        })
}

pub fn now_playing_embed(m: &mut CreateMessage, np: Metadata) {
    m.embed(|e| {
        e.title("Now playing");
        e.field("Title", np.title.clone().unwrap(), false);
        if let Some(t) = np.source_url {
            e.field("URL", t, false);
        }
        e.field("Duration", duration_format(np.duration), false);
        if let Some(t) = np.thumbnail {
            e.thumbnail(t);
        }

        e
    });
}

pub fn duration_format(duration: Option<Duration>) -> String {
    if let Some(d) = duration {
        if d != Duration::default() {
            return humantime::format_duration(d).to_string();
        }
    }
    "Live".to_string()
}
