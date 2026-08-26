use crate::bot::{
    CallbackReturn, app_state, callback_query_commands::CallbackCmd, utils::state_emoji,
};
use mpd::Query;
use std::sync::Arc;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::conf;

use super::utils::format_song;

pub async fn callback_query_handler(
    bot: Bot,
    q: CallbackQuery,
    conf: Arc<conf::Config>,
    app_state: Arc<app_state::AppState>,
) -> CallbackReturn {
    if q.from.id != conf.owner_id {
        bot.answer_callback_query(q.id)
            .show_alert(true)
            .text("You can't use this command")
            .await?;
        return Ok(());
    }
    let msg = q.message.unwrap();
    let mut mpd = conf.mpd.lock().await;
    let state_emoji = state_emoji(mpd.status()?);
    let current_song = mpd.currentsong()?;
    let buttons = InlineKeyboardMarkup {
        inline_keyboard: vec![
            vec![
                InlineKeyboardButton::callback("⏮️", CallbackCmd::Previous),
                InlineKeyboardButton::callback(state_emoji, CallbackCmd::PlayPause),
                InlineKeyboardButton::callback("⏭️", CallbackCmd::Next),
            ],
            vec![InlineKeyboardButton::callback(
                "🔀",
                CallbackCmd::ShuffleQueue,
            )],
        ],
    };
    let Some(data) = q.data else {
        return Ok(());
    };
    match CallbackCmd::from(data) {
        CallbackCmd::Add(cached_id) => {
            let Some(song) = app_state
                .get_track(cached_id)
                .and_then(|path| mpd.find(Query::new().and(mpd::Term::File, path), None).ok())
                .and_then(|results| results.into_iter().next())
            else {
                bot.answer_callback_query(q.id)
                    .text("Invalid search, might have expired")
                    .await?;
                return Ok(());
            };
            let current = mpd
                .currentsong()?
                .and_then(|song| song.place)
                .unwrap_or_default();

            mpd.insert(song, current.pos as usize + 1)?;
            bot.answer_callback_query(q.id).await?;
            bot.edit_message_text(msg.chat().id, msg.id(), "✅ Song added!")
                .await?;
        }
        CallbackCmd::PlayPause => {
            mpd.toggle_pause()?;
            bot.edit_message_text(msg.chat().id, msg.id(), format_song(current_song))
                .reply_markup(buttons)
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        CallbackCmd::Next => {
            mpd.next()?;
            bot.edit_message_text(msg.chat().id, msg.id(), format_song(current_song))
                .reply_markup(buttons)
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        CallbackCmd::Previous => {
            mpd.prev()?;
            bot.edit_message_text(msg.chat().id, msg.id(), format_song(current_song))
                .reply_markup(buttons)
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        CallbackCmd::ShuffleQueue => {
            mpd.shuffle(..)?;
            bot.answer_callback_query(q.id)
                .text("Shuffled queue!")
                .await?;
        }
        CallbackCmd::Play => {
            mpd.play()?;
            bot.answer_callback_query(q.id).text("Playing!").await?;
        }
        CallbackCmd::Pause => {
            mpd.pause(true)?;
            bot.answer_callback_query(q.id).text("Paused!").await?;
        }
        CallbackCmd::Stop => {
            mpd.stop()?;
            bot.answer_callback_query(q.id).text("Playing!").await?;
        }
        CallbackCmd::Clear => {
            mpd.clear()?;
            bot.answer_callback_query(q.id).text("Playing!").await?;
        }
        CallbackCmd::Unknown => {
            bot.answer_callback_query(q.id)
                .text("Unknown command!")
                .await?;
        }
    }
    Ok(())
}
