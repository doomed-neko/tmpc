use std::error::Error;

pub mod app_state;
pub mod callback_query_commands;
pub mod callback_query_handlers;
pub mod command_handlers;
pub mod commands;
pub mod schema;
pub mod utils;
pub type CallbackReturn = Result<(), Box<dyn Error + Send + Sync>>;
pub type HandlerResultErr = Box<dyn Error + Send + Sync>;
pub type HandlerResult = Result<(), HandlerResultErr>;
