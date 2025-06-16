use bot::{BotState, schema};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
mod bot;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let bot = Bot::new(include_str!("../token"));
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<BotState>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
