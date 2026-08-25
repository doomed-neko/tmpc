use std::{env, os::unix::net::UnixStream, path::PathBuf};

use mpd::Client;
use teloxide::types::UserId;

#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub owner_id: UserId,
    pub mpd: tokio::sync::Mutex<Client<UnixStream>>,
}
impl Config {
    pub fn parse() -> Result<Self, String> {
        let token =
            env::var("TMPC_TOKEN").map_err(|err| format!("Failed to read TMPC_TOKEN: {err}"))?;

        let owner_id = env::var("TMPC_OWNER")
            .map_err(|err| format!("Failed to read TMPC_OWNER: {err}"))?
            .parse::<u64>()
            .map(UserId)
            .map_err(|err| format!("Failed to parse owner id: {err}"))?;

        let mpd = tokio::sync::Mutex::new(
            Client::new(
                UnixStream::connect(
                    env::var("TMPC_MPD_SOCKET")
                        .map(PathBuf::from)
                        .map_err(|err| format!("Failed to read TMPC_MPD_SOCKET: {err}"))?,
                )
                .map_err(|err| format!("Failed to connect to socket: {err}"))?,
            )
            .map_err(|err| format!("Failed to create mpd client: {err}"))?,
        );
        Ok(Self {
            token,
            owner_id,
            mpd,
        })
    }
}
