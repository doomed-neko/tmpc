use std::sync::Arc;

use bot::{BotState, schema};
use log::error;
#[cfg(feature = "local")]
use reqwest::Url;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
mod bot;
mod conf;

// pub const MPD_SOCKET_PATH: &str = "/home/main/.config/mpd/socket";
// pub const REACTION_EMOJI: &str = "🍾";
// pub const OWNER_ID: UserId = UserId(8945237520);

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let conf = match conf::Config::parse().map(Arc::new) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to read config from env: {e}");
            return;
        }
    };
    let bot = Bot::new(&conf.token);
    #[cfg(feature = "local")]
    let bot = bot.set_api_url(Url::parse("http://127.0.0.1:8080").unwrap());
    Dispatcher::builder(bot, schema(conf.clone()))
        .dependencies(dptree::deps![InMemStorage::<BotState>::new(), conf])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
