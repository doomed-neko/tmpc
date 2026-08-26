use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Those are the available commands:"
)]
pub enum Commands {
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
}
