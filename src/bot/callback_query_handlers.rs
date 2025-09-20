use log::warn;
use mpd::{Client, Query};
use std::{error::Error, os::unix::net::UnixStream};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{MPD_SOCKET_PATH, OWNER_ID};

use super::utils::get_current;
pub type CallbackReturn = Result<(), Box<dyn Error + Send + Sync>>;

pub async fn callback_query_handler(bot: Bot, q: CallbackQuery) -> CallbackReturn {
    if q.from.id != OWNER_ID {
        bot.answer_callback_query(q.id)
            .show_alert(true)
            .text("You can't use this command")
            .await?;
        return Ok(());
    }
    let mut mpd = Client::new(UnixStream::connect(MPD_SOCKET_PATH)?)?;
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
    let msg = q.message.unwrap();
    let Some(mut data) = q.data else {
        return Ok(());
    };
    let cmd = data.pop().unwrap();
    match cmd {
        'i' => {
            let db = sled::open("DB")?.open_tree("search")?;
            let Some(text) = db.get(&data)? else {
                return Ok(());
            };
            let Some(song) = mpd
                .find(
                    Query::new().and(mpd::Term::File, String::from_utf8_lossy(&text)),
                    None,
                )?
                .into_iter()
                .next()
            else {
                return Ok(());
            };
            let current = mpd
                .currentsong()?
                .unwrap_or_default()
                .place
                .unwrap_or_default();

            mpd.insert(song, current.pos as usize + 1)?;
            bot.answer_callback_query(q.id).await?;
            bot.edit_message_text(msg.chat().id, msg.id(), "✅ Song added!")
                .await?;
        }
        'p' => {
            mpd.toggle_pause()?;
            bot.edit_message_text(msg.chat().id, msg.id(), get_current(&mut mpd))
                .reply_markup(buttons)
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        'n' => {
            mpd.next()?;
            bot.edit_message_text(msg.chat().id, msg.id(), get_current(&mut mpd))
                .reply_markup(buttons)
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        'P' => {
            mpd.prev()?;
            bot.edit_message_text(msg.chat().id, msg.id(), get_current(&mut mpd))
                .reply_markup(buttons)
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        's' => {
            mpd.shuffle(..)?;
            bot.answer_callback_query(q.id)
                .text("Shuffled queue!")
                .await?;
        }
        a => {
            warn!("Unhandled callback query command: {a}");
        }
    }
    Ok(())
}
