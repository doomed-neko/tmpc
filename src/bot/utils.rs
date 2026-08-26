use std::{os::unix::net::UnixStream, sync::Arc};

use mpd::Client;
use teloxide::{prelude::*, types::ReactionType};

use crate::{bot::HandlerResult, conf};

pub fn get_current(mpd: &mut Client<UnixStream>) -> String {
    let song = mpd
        .currentsong()
        .ok()
        .unwrap_or_default()
        .unwrap_or_default();

    let title = song.title.unwrap_or("Unknown".into());
    let artist = song.artist.unwrap_or("Unknown".into());
    let album = song
        .tags
        .into_iter()
        .filter_map(|(name, val)| {
            if name.to_lowercase().as_str() == "album" {
                Some(val)
            } else {
                None
            }
        })
        .collect::<String>();

    format!("🎵 {title}\n👤 {artist}\n💿 {album}")
}

pub async fn react_to_msg(bot: Bot, msg: Message, conf: Arc<conf::Config>) -> HandlerResult {
    bot.set_message_reaction(msg.chat.id, msg.id)
        .reaction(vec![ReactionType::Emoji {
            emoji: conf.reaction_emoji.clone(),
        }])
        .await?;
    Ok(())
}
