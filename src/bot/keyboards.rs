use mpd::Status;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::bot::{callback_query_commands::CallbackCmd, utils::state_emoji};

pub fn dashboard_keyboard(player_status: Status) -> InlineKeyboardMarkup {
    let state_emoji = state_emoji(player_status);
    InlineKeyboardMarkup {
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
    }
}

pub fn inline_search_keyboard(id: u32) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup {
        inline_keyboard: vec![
            vec![InlineKeyboardButton::callback(
                "➕ Add to queue",
                CallbackCmd::Add(id),
            )],
            vec![InlineKeyboardButton::callback(
                "🔺 Play Now",
                CallbackCmd::PlayNow(id),
            )],
        ],
    }
}
