use std::{env, os::unix::net::UnixStream, path::PathBuf};

use mpd::Client;
use teloxide::types::UserId;

#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub owner_id: UserId,
    pub reaction_emoji: String,
    pub admin_mode: bool,
    pub socket_path: PathBuf,
}
impl Config {
    pub fn mpd(&self) -> Result<Client<UnixStream>, String> {
        let socket = UnixStream::connect(&self.socket_path)
            .map_err(|err| format!("Failed to connect to socket: {err}"))?;
        Client::new(socket).map_err(|err| format!("Failed to create mpd client: {err}"))
    }
    pub fn parse() -> Result<Self, String> {
        let token =
            env::var("TMPC_TOKEN").map_err(|err| format!("Failed to read TMPC_TOKEN: {err}"))?;
        let reaction_emoji = env::var("TMPC_REACTION_EMOJI").unwrap_or("🍾".into());

        let admin_mode = env::var("TMPC_ADMIN_MODE")
            .map(|val| val != "0")
            .unwrap_or(false);

        let owner_id = env::var("TMPC_OWNER")
            .map_err(|err| format!("Failed to read TMPC_OWNER: {err}"))?
            .parse::<u64>()
            .map(UserId)
            .map_err(|err| format!("Failed to parse owner id: {err}"))?;

        let socket_path = env::var("TMPC_MPD_SOCKET")
            .map(PathBuf::from)
            .map_err(|err| format!("Failed to read TMPC_MPD_SOCKET: {err}"))?;
        Ok(Self {
            token,
            owner_id,
            admin_mode,
            reaction_emoji,
            socket_path,
        })
    }
}
