use std::sync::Arc;

use bot::schema::{BotState, schema};
use log::error;
#[cfg(feature = "local")]
use reqwest::Url;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

use crate::bot::app_state;
mod bot;
mod conf;

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
    let app_state = Arc::new(app_state::AppState::new());
    let bot = Bot::new(&conf.token);
    #[cfg(feature = "local")]
    let bot = bot.set_api_url(Url::parse("http://127.0.0.1:8080").unwrap());
    Dispatcher::builder(bot, schema(conf.clone()))
        .dependencies(dptree::deps![
            InMemStorage::<BotState>::new(),
            conf,
            app_state
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
