use log::warn;
use mpd::{Client, Query};
use std::{error::Error, fs, os::unix::net::UnixStream, path::PathBuf};
use teloxide::prelude::*;

use crate::MPD_SOCKET_PATH;
pub type CallbackReturn = Result<(), Box<dyn Error + Send + Sync>>;

pub async fn callback_query_handler(bot: Bot, q: CallbackQuery) -> CallbackReturn {
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
    let Some(mut data) = q.data else {
        return Ok(());
    };
    let cmd = data.pop().unwrap();
    match cmd {
        'i' => {
            let mut path = PathBuf::from("uuid/");
            path.push(&data);

            let text = String::from_utf8_lossy(&fs::read(&path)?).to_string();
            let Some(song) = mpd
                .find(Query::new().and(mpd::Term::File, text), None)?
                .into_iter()
                .next()
            else {
                return Ok(());
            };
            let Some(current) = mpd.currentsong()? else {
                return Ok(());
            };
            let Some(current) = current.place else {
                return Ok(());
            };
            mpd.insert(song, current.pos as usize + 1)?;
            bot.answer_callback_query(q.id).await?;
            let msg = q.message.unwrap();
            bot.edit_message_text(msg.chat().id, msg.id(), "✅ Song added!")
                .await?;
            fs::read_dir(path.parent().unwrap()).unwrap().for_each(|x| {
                if let Ok(entry) = x {
                    let _ = fs::remove_file(entry.path());
                }
            });
        }
        'n' => {}
        a => {
            warn!("Unhandled callback query command: {a}");
        }
    }
    Ok(())
}
