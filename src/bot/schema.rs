use std::sync::Arc;

use command_handlers::*;
use teloxide::{
    dispatching::{
        UpdateFilterExt, UpdateHandler,
        dialogue::{self, InMemStorage},
    },
    dptree::{self, case},
    filter_command,
    types::{Message, Update},
};

use crate::{
    bot::{
        callback_query_handlers::callback_query_handler, command_handlers, commands::Commands,
        utils::react_to_msg,
    },
    conf,
};
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
        .branch(case![Commands::Stats].endpoint(stats));
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
