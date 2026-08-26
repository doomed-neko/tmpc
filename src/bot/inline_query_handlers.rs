use std::sync::Arc;

use mpd::{Query, Term};
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InlineQueryResultsButton,
        InlineQueryResultsButtonKind, InputMessageContent, InputMessageContentText,
    },
};

use crate::{
    bot::{
        CallbackReturn, app_state,
        callback_query_commands::CallbackCmd,
        keyboards::{dashboard_keyboard, inline_search_keyboard},
        utils::format_song,
    },
    conf,
};

pub async fn inline_query_handler(
    bot: Bot,
    q: InlineQuery,
    conf: Arc<conf::Config>,
    app_state: Arc<app_state::AppState>,
) -> CallbackReturn {
    let mut mpd = conf.mpd()?;
    let artist_tag = "Artist";
    let title_tag = "Title";
    let user_query = q.query.trim();
    let mut query = Query::new();
    match user_query.split_once("-") {
        Some((artist, title)) => query
            .and(mpd::Term::Tag(artist_tag.into()), artist)
            .and(mpd::Term::Tag(title_tag.into()), title),
        None => query.and(Term::Any, user_query),
    };

    let mut results = mpd.search(&query, None)?;
    results.truncate(50);
    let results = results
        .into_iter()
        .map(|song| {
            let message_content = InputMessageContentText::new(format_song(Some(song.clone())));
            let id = app_state.cache_track(song.file);
            let unknown = String::from("Unkonwn");
            let title = song.title.unwrap_or(unknown.clone());
            let artist = song.artist.unwrap_or(unknown);
            InlineQueryResult::Article(
                InlineQueryResultArticle::new(
                    [&artist, "-", &title].join(""),
                    &title,
                    InputMessageContent::Text(message_content),
                )
                .description(format!("{title} - {artist}"))
                .reply_markup(inline_search_keyboard(id)),
            )
        })
        .take(50)
        .collect::<Vec<_>>();
    let button = InlineQueryResultsButton {
        text: "Show controls".into(),
        kind: InlineQueryResultsButtonKind::StartParameter("controls".into()),
    };
    bot.answer_inline_query(q.id, results)
        .button(button)
        .await?;

    Ok(())
}
pub async fn inline_msg_handler(
    bot: Bot,
    q: CallbackQuery,
    conf: Arc<conf::Config>,
    app_state: Arc<app_state::AppState>,
) -> CallbackReturn {
    let mut mpd = conf.mpd()?;
    let current_song = mpd.currentsong().unwrap_or_default();
    let Some(msg_id) = q.inline_message_id else {
        return Ok(());
    };
    let Some(data) = q.data else {
        return Ok(());
    };
    match CallbackCmd::from(data) {
        CallbackCmd::PlayNow(cached_id) => {
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
            mpd.next()?;
            bot.answer_callback_query(q.id).await?;
            bot.edit_message_text_inline(msg_id, "✅ Song playing now")
                .await?;
        }
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
            bot.edit_message_text_inline(msg_id, "✅ Song added!")
                .await?;
        }
        CallbackCmd::PlayPause => {
            mpd.toggle_pause()?;
            bot.edit_message_text_inline(msg_id, format_song(current_song))
                .reply_markup(dashboard_keyboard(mpd.status()?))
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        CallbackCmd::Previous => {
            mpd.prev()?;
            bot.edit_message_text_inline(msg_id, format_song(current_song))
                .reply_markup(dashboard_keyboard(mpd.status()?))
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        CallbackCmd::Next => {
            mpd.next()?;
            bot.edit_message_text_inline(msg_id, format_song(current_song))
                .reply_markup(dashboard_keyboard(mpd.status()?))
                .await?;
            bot.answer_callback_query(q.id).await?;
        }
        _ => {}
    }

    Ok(())
}
