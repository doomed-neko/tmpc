use std::sync::Arc;

use mpd::Song;
use teloxide::{prelude::*, types::ReactionType};

use crate::conf;

pub fn format_song(song: Option<Song>) -> String {
    if let Some(song) = song {
        let unknown = String::from("Unknown");
        let title = song.title.unwrap_or(unknown.clone());
        let artist = song.artist.unwrap_or(unknown.clone());
        let album = song
            .tags
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("album"))
            .map(|(_, val)| val)
            .unwrap_or(unknown.clone());

        format!("🎵 {title}\n👤 {artist}\n💿 {album}")
    } else {
        String::from("🎵 No song playing right now")
    }
}

pub async fn react_to_msg(bot: Bot, msg: Message, conf: Arc<conf::Config>) {
    bot.set_message_reaction(msg.chat.id, msg.id)
        .reaction(vec![ReactionType::Emoji {
            emoji: conf.reaction_emoji.clone(),
        }])
        .await
        .ok();
}
