use std::{error::Error, sync::Arc};

use callback_query_handlers::callback_query_handler;
use command_handlers::*;
use teloxide::{
    dispatching::{
        UpdateFilterExt, UpdateHandler,
        dialogue::{self, InMemStorage},
    },
    dptree::{self, case},
    filter_command,
    macros::BotCommands,
    types::{Message, Update},
};

use crate::{bot::utils::react_to_msg, conf};

mod callback_query_handlers;
mod command_handlers;
mod utils;
pub type CallbackReturn = Result<(), Box<dyn Error + Send + Sync>>;
pub type HandlerResultErr = Box<dyn Error + Send + Sync>;
pub type HandlerResult = Result<(), HandlerResultErr>;

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
    #[command(description = "Play/Pause music", aliases=["p"])]
    Play,
    #[command(description = "Switch to next track", aliases=["n"])]
    Next,
    #[command(description = "Switch to previous track")]
    Prev,
    #[command(description = "Show information about current song", aliases=["np"])]
    Current,
    #[command(description = "Show songs in the queue", aliases=["q"])]
    Queue,
    #[command(description = "Add a song from youtube", aliases=["yt"])]
    AddYt,
    #[command(description = "Search in the db", aliases=["s"])]
    Search(String),
    #[command(description = "Add random songs", aliases=["rand"])]
    AddRand(String),
    #[command(description = "Add all songs to queue", aliases=["all"])]
    AddAll,
    #[command(description = "Add an audio file to queue", aliases=["file"])]
    AddFile,
    #[command(description = "Clear the queue")]
    Clear,
    #[command(description = "Shuffle the queue")]
    Shuffle,
    #[command(description = "Show DB stats")]
    Stats,
    #[command(description = "Save file id for song")]
    Save,
}

#[derive(Default, Clone, Copy)]
pub enum BotState {
    #[default]
    Start,
}
pub fn schema(
    conf: Arc<conf::Config>,
) -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let react_hook = dptree::inspect_async(react_to_msg);
    let commands_branch =
        dptree::filter(move |msg: Message| msg.from.is_some_and(|x| x.id == conf.owner_id))
            .branch(case![Commands::Play].endpoint(play))
            .branch(case![Commands::Next].endpoint(next))
            .branch(case![Commands::Clear].endpoint(clear))
            .branch(case![Commands::AddYt].endpoint(add_yt))
            .branch(case![Commands::AddRand(amount)].endpoint(add_rand))
            .branch(case![Commands::AddAll].endpoint(add_all))
            .branch(case![Commands::Shuffle].endpoint(shuffle))
            .branch(case![Commands::AddFile].endpoint(add_file))
            .branch(case![Commands::Search(query)].endpoint(search))
            .branch(case![Commands::Prev].endpoint(prev));

    let public_commands_branch = dptree::entry()
        .branch(case![Commands::Start].endpoint(start))
        .branch(case![Commands::Help].endpoint(help))
        .branch(case![Commands::Current].endpoint(curr))
        .branch(case![Commands::Queue].endpoint(queue))
        .branch(case![Commands::Stats].endpoint(stats))
        .branch(case![Commands::Save].endpoint(save));
    let cmd_handler = filter_command::<Commands, _>()
        .chain(react_hook)
        .branch(commands_branch)
        .branch(public_commands_branch);
    let msg_handler = Update::filter_message().branch(cmd_handler);
    let callback_query_handler = Update::filter_callback_query().endpoint(callback_query_handler);
    dialogue::enter::<Update, InMemStorage<BotState>, BotState, _>()
        .branch(msg_handler)
        .branch(callback_query_handler)
        .endpoint(async || Ok(()))
}
