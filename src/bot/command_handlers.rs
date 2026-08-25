use log::{error, info};
use mpd::{Client, Query, Song, search::Window};
use rand::prelude::IndexedRandom;
use std::{
    env::temp_dir, error::Error, fs::DirBuilder, os::unix::net::UnixStream, path::PathBuf,
    process::Command, time::Duration,
};
#[cfg(feature = "local")]
use teloxide::types::{FileId, InputFile};
use teloxide::{
    net::Download,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ReactionType, ReplyParameters},
    utils::command::BotCommands,
};
use tokio::fs::File as AsyncFile;

use crate::{MPD_SOCKET_PATH, REACTION_EMOJI};

use super::{Commands, utils::get_current};
type HandlerResultErr = Box<dyn Error + Send + Sync>;
type HandlerResult = Result<(), HandlerResultErr>;

pub async fn start(bot: Bot, msg: Message) -> HandlerResult {
    let buttons = vec![
        vec![InlineKeyboardButton::callback("play/pause", "p")],
        vec![
            InlineKeyboardButton::callback("prev", "P"),
            InlineKeyboardButton::callback("next", "n"),
        ],
        vec![InlineKeyboardButton::callback("shuffle", "s")],
    ];
    let buttons = InlineKeyboardMarkup {
        inline_keyboard: buttons,
    };

    let text = {
        let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
        get_current(&mut mpd)
    };
    bot.send_message(msg.chat.id, text)
        .reply_markup(buttons)
        .await?;
    Ok(())
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Commands::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn clear(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match mpd.clear() {
        Ok(_) => {
            info!("Cleared queue");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: REACTION_EMOJI.into(),
                }])
                .await?;
        }
        Err(t) => {
            error!("{t}");
        }
    };

    Ok(())
}

pub async fn play(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match mpd.toggle_pause() {
        Ok(_) => {
            info!("Toggled playback");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: REACTION_EMOJI.into(),
                }])
                .await?;
        }
        Err(t) => {
            error!("{t}");
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
                    emoji: REACTION_EMOJI.into(),
                }])
                .await?;
        }
        Err(t) => {
            error!("{t}");
        }
    };
    let text = get_current(&mut mpd);
    bot.send_message(msg.chat.id, text)
        .reply_parameters(ReplyParameters {
            message_id: msg.id,
            ..Default::default()
        })
        .await?;

    Ok(())
}

pub async fn prev(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match mpd.prev() {
        Ok(_) => {
            info!("Prev song");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: REACTION_EMOJI.into(),
                }])
                .await?;
        }
        Err(t) => {
            error!("{t}");
        }
    };

    let Some(song) = mpd.currentsong()? else {
        return Ok(());
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

    let text = format!("🎵 {title}\n👤 {artist}\n💿 {album}");
    bot.send_message(msg.chat.id, text)
        .reply_parameters(ReplyParameters {
            message_id: msg.id,
            ..Default::default()
        })
        .await?;

    Ok(())
}

pub async fn shuffle(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    match mpd.shuffle(..) {
        Ok(_) => {
            info!("Prev song");
            bot.set_message_reaction(msg.chat.id, msg.id)
                .reaction(vec![ReactionType::Emoji {
                    emoji: REACTION_EMOJI.into(),
                }])
                .await?;
        }
        Err(t) => {
            error!("{t}");
        }
    };

    Ok(())
}

pub async fn curr(bot: Bot, msg: Message) -> HandlerResult {
    info!("Current song info sent");
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let song = match mpd.currentsong()? {
        Some(t) => t,
        None => {
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

    let text = format!("🎵 {title}\n👤 {artist}\n💿 {album}");
    bot.send_message(msg.chat.id, text).await?;

    #[cfg(feature = "local")]
    {
        tokio::spawn(async move {
            let file_name = song.file;
            let Some(file_path) = mpd
                .mounts()?
                .into_iter()
                .filter_map(|x| {
                    let path = format!("{}/{}", x.storage, file_name);
                    let path_b = PathBuf::from(&file_name);
                    if path_b.is_absolute() && path_b.exists() {
                        return Some(file_name.clone());
                    }
                    if PathBuf::from(&path).exists() {
                        return Some(path);
                    }
                    None
                })
                .next()
            else {
                bot.send_message(msg.chat.id, "Something wrong happened")
                    .await?;
                return Ok::<_, HandlerResultErr>(());
            };
            info!("Got file path");

            let db = sled::open("DB")?.open_tree("file_ids")?;
            if let Some(sticker) = db.get(&file_name)? {
                let sticker = String::from_utf8_lossy(&sticker).to_string();
                info!("Sending via file id");
                bot.send_audio(msg.chat.id, InputFile::file_id(FileId(sticker)))
                    .performer(artist)
                    .title(title)
                    .reply_parameters(ReplyParameters {
                        message_id: msg.id,
                        ..Default::default()
                    })
                    .await?;
            } else {
                info!("Sending via file");
                bot.send_audio(msg.chat.id, InputFile::file(file_path))
                    .performer(artist)
                    .title(title)
                    .reply_parameters(ReplyParameters {
                        message_id: msg.id,
                        ..Default::default()
                    })
                    .await
                    .map(|x| {
                        let file_id = x.audio().unwrap().file.id.clone().0;
                        db.insert(&file_name, file_id.as_bytes())
                    })??;
            }
            Ok(())
        })
        .await??;
    }
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
            error!("{t}");
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
    let text = format!("🎛Queue length: {queue_len}\n\n{queue_formatted}");
    info!("Queue info sent");
    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}

pub async fn add_yt(bot: Bot, msg: Message) -> HandlerResult {
    let Some(Some(url)) = msg
        .reply_to_message()
        .map(|x| x.text().map(|x| x.to_string()))
    else {
        bot.send_message(
            msg.chat.id,
            "No url provided\nReply to a message with a video url to add it to queue",
        )
        .await?;
        return Ok(());
    };

    let url = if url.contains("youtu.be/") {
        url.replace("?", "&")
            .replace("youtu.be/", "youtube.com/watch?v=")
    } else {
        url
    };

    bot.set_message_reaction(msg.chat.id, msg.id)
        .reaction(vec![ReactionType::Emoji {
            emoji: REACTION_EMOJI.into(),
        }])
        .await?;
    bot.send_message(
        msg.chat.id,
        "⏳ Downloading video... \nPlease wait, this might take a minute",
    )
    .reply_parameters(ReplyParameters {
        message_id: msg.id,
        ..Default::default()
    })
    .await?;

    match Command::new("rmpc")
        .arg("addyt")
        .arg("-p")
        .arg("+0")
        .arg(url)
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
    if query.is_empty() {
        bot.send_message(
            msg.chat.id,
            "No search query\nUsage:\n    `/search enter sandman`",
        )
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
        return Ok(());
    }
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let db = sled::open("DB")?.open_tree("search")?;
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
            db.insert(&id, f.file.as_bytes()).ok();
            // let _ = fs::write(path, f.file);
            (text, id)
        })
        .map(|(text, id)| vec![InlineKeyboardButton::callback(text, format!("{id}i"))])
        .collect::<Vec<_>>();

    if !buttons.is_empty() {
        let btns = buttons.len();
        let kbd = InlineKeyboardMarkup::new(buttons);
        bot.send_message(
            msg.chat.id,
            format!("{btns} resluts found. Tap on a button to add to queue:"),
        )
        .reply_markup(kbd)
        .await?;
    } else {
        bot.send_message(msg.chat.id, "No results found!").await?;
    }

    Ok(())
}

pub async fn add_rand(bot: Bot, msg: Message, amount: String) -> HandlerResult {
    let amount = if amount.is_empty() {
        "1".into()
    } else {
        amount
    };
    let Ok(amount) = amount.parse::<usize>() else {
        bot.send_message(msg.chat.id, "Invalid number").await?;
        return Ok(());
    };

    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH).unwrap()).unwrap();
    let mut current = mpd
        .currentsong()?
        .unwrap_or_default()
        .place
        .unwrap_or_default()
        .pos as usize;
    let songs = mpd.listall()?;
    let songs = songs.choose_multiple(&mut rand::rng(), amount).cloned();
    for song in songs {
        mpd.insert(song, current)?;
        current += 1;
    }

    bot.send_message(
        msg.chat.id,
        format!("Successfully added {amount} random songs to the queue"),
    )
    .await?;
    Ok(())
}
pub async fn add_all(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    println!("test");
    let stats = mpd.stats()?;
    println!("test");
    let all_songs = Song {
        file: "/".into(),
        ..Default::default()
    };
    mpd.push(all_songs)?;
    println!("test");
    mpd.toggle_pause()?;
    println!("test");
    bot.send_message(
        msg.chat.id,
        format!("Successfully added {} songs to the queue", stats.songs),
    )
    .await?;

    Ok(())
}

pub async fn add_file(bot: Bot, msg: Message) -> HandlerResult {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    {
        let mut tmp = temp_dir();
        tmp.push("tmpc");
        if !tmp.exists() {
            DirBuilder::new().create(tmp)?;
        }
    }

    let Some(replied_to) = msg.reply_to_message() else {
        bot.send_message(msg.chat.id, "No message replied to")
            .await?;
        return Ok(());
    };
    let Some(audio) = replied_to.audio() else {
        bot.send_message(msg.chat.id, "Message does not contain audio!")
            .await?;
        return Ok(());
    };

    let file = bot.get_file(audio.file.id.clone()).await?;
    if file.size > 20 * 1024 * 1024 {
        bot.send_message(
            msg.chat.id,
            "❌ File too big, can't download files larger than 20MB",
        )
        .await?;
        return Ok(());
    }
    let file_path = {
        let Some(file_name) = &audio.file_name else {
            bot.send_message(msg.chat.id, "❌ Invalid audio file found")
                .await?;
            return Ok(());
        };
        let mut path = temp_dir();
        path.push("tmpc");
        path.push(file_name);
        path
    };
    let song = Song {
        file: file_path.to_string_lossy().to_string(),
        ..Default::default()
    };

    let msg_id = msg.id;
    let chat_id = msg.chat.id;
    if !file_path.exists() {
        bot.send_message(
            msg.chat.id,
            "⏳ Downloading the file, this might take some time",
        )
        .await?;
        let mut output_file = AsyncFile::create(file_path.clone()).await?;
        if let Err(e) = bot.download_file(&file.path, &mut output_file).await {
            match e {
                teloxide::DownloadError::Network(error) => {
                    error!("{error}");
                    bot.send_message(
                        chat_id,
                        "❌ Failed to download file due to a Network Error, try again",
                    )
                    .reply_parameters(ReplyParameters {
                        message_id: msg_id,
                        ..Default::default()
                    })
                    .await?;
                }
                teloxide::DownloadError::Io(error) => {
                    error!("{error}");
                    bot.send_message(chat_id, "❌ Failed to download file due to an I/O Error")
                        .reply_parameters(ReplyParameters {
                            message_id: msg_id,
                            ..Default::default()
                        })
                        .await?;
                }
            };
            return Ok(());
        }
    }
    bot.send_message(chat_id, "✅Song added to queue!")
        .reply_parameters(ReplyParameters {
            message_id: msg_id,
            ..Default::default()
        })
        .await?;
    let current = mpd.currentsong()?.unwrap_or_default();
    let pos = current.place.unwrap_or_default().pos as usize;
    if mpd.insert(&song, pos + 1).is_err() {
        mpd.push(song)?;
    };

    Ok(())
}

pub async fn save(bot: Bot, msg: Message) -> HandlerResult {
    let db = sled::open("DB")?.open_tree("file_ids")?;
    let Some(replied_to) = msg.reply_to_message() else {
        bot.send_message(msg.chat.id, "No message replied to")
            .await?;
        return Ok(());
    };
    let Some(audio) = replied_to.audio() else {
        bot.send_message(msg.chat.id, "Message does not contain audio!")
            .await?;
        return Ok(());
    };
    let Some(file_name) = audio.file_name.clone() else {
        bot.send_message(msg.chat.id, "Invalid audio file").await?;
        return Ok(());
    };

    let FileId(file_id) = audio.file.id.clone();
    db.insert(file_name, file_id.as_bytes())?;

    bot.send_message(msg.chat.id, "Done! file saved and can be reused now <3")
        .reply_parameters(ReplyParameters {
            message_id: msg.id,
            ..Default::default()
        })
        .await?;

    Ok(())
}
