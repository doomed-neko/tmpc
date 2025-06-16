use log::{error, info};
use mpd::{Client, Query, search::Window};
use std::{
    error::Error,
    fs,
    os::unix::net::UnixStream,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, LinkPreviewOptions, ReactionType},
    utils::command::BotCommands,
};

use crate::MPD_SOCKET_PATH;

use super::Commands;
type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

pub async fn start(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Commands::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Commands::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn clear(bot: Bot, msg: Message) -> HandlerResult {
    let mut conn = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match conn.clear() {
        Ok(_) => {
            info!("Cleared queue");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "❤️".into()
                }])
                .await?;
        }
        Err(t) => {
            error!("{}", t);
        }
    };

    Ok(())
}

pub async fn play(bot: Bot, msg: Message) -> HandlerResult {
    match Command::new("rmpc").arg("togglepause").output() {
        Ok(_) => {
            info!("Toggled playback");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "❤️".into()
                }])
                .await?;
        }
        Err(t) => {
            error!("{}", t);
        }
    };

    Ok(())
}

pub async fn next(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match mpd.next() {
        Ok(_) => {
            info!("Next song");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "❤️".into()
                }])
                .await?;
        }
        Err(t) => {
            error!("{}", t);
        }
    };

    Ok(())
}

pub async fn prev(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match mpd.prev() {
        Ok(_) => {
            info!("Prev song");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "❤️".into()
                }])
                .await?;
        }
        Err(t) => {
            error!("{}", t);
        }
    };

    Ok(())
}

pub async fn shuffle(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match mpd.shuffle(..) {
        Ok(_) => {
            info!("Prev song");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: "❤️".into()
                }])
                .await?;
        }
        Err(t) => {
            error!("{}", t);
        }
    };

    Ok(())
}

pub async fn curr(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let song = match mpd.currentsong()? {
        Some(t) => t,
        None => {
            info!("Current song info sent");
            bot.send_message(msg.chat.id, "No song playing right now")
                .await?;
            return Ok(());
        }
    };
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

    let text = format!("🎵{title}\n👤{artist}\n💿{album}");
    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}

pub async fn queue(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let Some(current) = mpd.currentsong()? else {
        return Ok(());
    };
    let Some(current) = current.place else {
        return Ok(());
    };
    let current = current.pos as usize;
    let queue = match mpd.queue() {
        Ok(t) => t,
        Err(t) => {
            error!("{}", t);
            return Ok(());
        }
    };
    let queue = Vec::from(&queue[current..]);
    let queue_len = queue.len();
    let mut queue_formatted = queue
        .into_iter()
        .map(|f| {
            [
                "🎵".into(),
                f.title.unwrap_or("Unknown".into()),
                "-".into(),
                f.artist.unwrap_or("Unknown".into()),
            ]
            .join(" ")
        })
        .take(20)
        .collect::<Vec<String>>()
        .join("\n\n");
    if queue_formatted.is_empty() {
        queue_formatted = "No song in queue".into();
    }
    let text = format!("🎛Queue length: {}\n{queue_formatted}", queue_len);
    info!("Queue info sent");
    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}

pub async fn add_yt(bot: Bot, msg: Message, url: String) -> HandlerResult {
    if url.is_empty() {
        bot.send_message(msg.chat.id, "No url provided\nUsage:\n        /addyt <URL>\nExample:\n        /addyt https://www.youtube.com/watch?v=3ehWXsLtPoY")
            .link_preview_options(LinkPreviewOptions{ is_disabled: true,url:None, prefer_small_media:true, prefer_large_media: false, show_above_text:true })
        .await?;
        return Ok(());
    }
    bot.set_message_reaction(msg.chat.id, msg.id)
        .reaction(vec![ReactionType::Emoji {
            emoji: "❤️".into()
        }])
        .await?;
    let url = if url.contains("youtu.be/") {
        url.replace("?", "&")
            .replace("youtu.be/", "youtube.com/watch?v=")
    } else {
        url
    };
    match Command::new("fish")
        .arg("-c")
        .arg(format!("rmpc addyt {url}"))
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
    {
        Ok(mut f) => {
            if f.wait()?.success() {
                bot.send_message(msg.chat.id, "✅ Added youtube song to queue!")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "❌ Failed to add song".to_string())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            }
        }
        Err(e) => {
            bot.send_message(
                msg.chat.id,
                format!("❌ Failed to add song:\n```\n{e}\n```"),
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
        }
    };
    Ok(())
}

fn humanize_duration(dur: Duration) -> String {
    use humanize_duration::Truncate;
    use humanize_duration::prelude::DurationExt;
    dur.human(Truncate::Second).to_string()
}

pub async fn stats(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let stats = mpd.stats()?;
    let artists = stats.artists;
    let albums = stats.albums;
    let songs = stats.songs;
    let db_playtime = humanize_duration(stats.db_playtime);
    let text = format!(
        r#"👤 Number of artists: {artists}
💿Number of albums: {albums}
🎵Number of songs: {songs}

total duration: {db_playtime}"#
    );

    info!("Sent db stats");
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

pub async fn search(bot: Bot, msg: Message, query: String) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let mut q = Query::new();
    let query_mpd = q.and(mpd::Term::Tag("Title".into()), &query);
    let buttons = mpd
        .search(query_mpd, Window::from((0, 95)))?
        .into_iter()
        .map(|f| {
            let title = f.title.unwrap_or("Unknown".into());
            let artist = f.artist.unwrap_or("Unknown".into());
            let text = [artist, "-".into(), title].join(" ");
            let id = uuid::Uuid::now_v7().as_simple().to_string();
            let mut path = PathBuf::from("uuid/");
            path.push(&id);
            let _ = fs::write(path, f.file);
            (text, id)
        })
        .map(|(text, id)| vec![InlineKeyboardButton::callback(text, format!("{}i", id))])
        .collect::<Vec<_>>();

    if !buttons.is_empty() {
        let btns = buttons.len();
        let kbd = InlineKeyboardMarkup::new(buttons);
        bot.send_message(
            msg.chat.id,
            format!("{} resluts found. Tap on a button to add to queue:", btns),
        )
        .reply_markup(kbd)
        .await?;
    } else {
        bot.send_message(msg.chat.id, "No results found!").await?;
    }

    Ok(())
}

#[allow(unused_variables)]
pub async fn add_rand(bot: Bot, msg: Message, amount: String) -> HandlerResult {
    Command::new("rmpc")
        .arg("addrandom")
        .arg("song")
        .arg(&amount)
        .output()?;
    bot.send_message(
        msg.chat.id,
        format!("Successfully added {} random songs to the queue", amount),
    )
    .await?;
    Ok(())
}
pub async fn add_all(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let stats = mpd.stats()?;
    Command::new("rmpc").arg("add").arg("/").output()?;
    bot.send_message(
        msg.chat.id,
        format!("Successfully added {} songs to the queue", stats.songs),
    )
    .await?;

    Ok(())
}
