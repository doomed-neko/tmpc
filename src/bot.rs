use callback_query_handlers::callback_query_handler;
use command_handlers::*;
use teloxide::{
    dispatching::{
        UpdateFilterExt, UpdateHandler,
        dialogue::{self, InMemStorage},
    },
    dptree::case,
    filter_command,
    macros::BotCommands,
    types::Update,
};

mod callback_query_handlers;
mod command_handlers;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Those are the available commands:"
)]
enum Commands {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show this help")]
    Help,
    #[command(description = "Play/Pause music")]
    Play,
    #[command(description = "Switch to next track")]
    Next,
    #[command(description = "Switch to previous track")]
    Prev,
    #[command(description = "Show information about current song")]
    Current,
    #[command(description = "Show songs in the queue")]
    Queue,
    #[command(description = "Add a song from youtube")]
    AddYt(String),
    #[command(description = "Show DB stats")]
    Stats,
    #[command(description = "Search in the db")]
    Search(String),
    #[command(description = "Add random songs")]
    AddRand(String),
    #[command(description = "Add all songs to queue")]
    AddAll,
    #[command(description = "Clear the queue")]
    Clear,
    #[command(description = "Shuffle the queue")]
    Shuffle,
}

#[derive(Default, Clone, Copy)]
pub enum BotState {
    #[default]
    Start,
}
pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let cmd_handler = filter_command::<Commands, _>()
        .branch(case![Commands::Start].endpoint(start))
        .branch(case![Commands::Help].endpoint(help))
        .branch(case![Commands::Play].endpoint(play))
        .branch(case![Commands::Next].endpoint(next))
        .branch(case![Commands::Prev].endpoint(prev))
        .branch(case![Commands::Current].endpoint(curr))
        .branch(case![Commands::Queue].endpoint(queue))
        .branch(case![Commands::Stats].endpoint(stats))
        .branch(case![Commands::Clear].endpoint(clear))
        .branch(case![Commands::Search(query)].endpoint(search))
        .branch(case![Commands::AddRand(amount)].endpoint(add_rand))
        .branch(case![Commands::AddAll].endpoint(add_all))
        .branch(case![Commands::Shuffle].endpoint(shuffle))
        .branch(case![Commands::AddYt(url)].endpoint(add_yt));
    let callback_query_handler = Update::filter_callback_query().endpoint(callback_query_handler);
    let msg_handler = Update::filter_message().branch(cmd_handler);
    dialogue::enter::<Update, InMemStorage<BotState>, BotState, _>()
        .branch(msg_handler)
        .branch(callback_query_handler)
}
