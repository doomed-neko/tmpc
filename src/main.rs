use std::env;

use bot::{BotState, schema};
use log::error;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
mod bot;

pub const MPD_SOCKET_PATH: &str = "/home/pasta/.config/mpd/socket";
pub const REACTION_EMOJI: &str = "🍾";

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let Ok(token) = env::var("TMPC_TOKEN") else {
        error!("No token defined");
        return;
    };
    let bot = Bot::new(token);
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<BotState>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
