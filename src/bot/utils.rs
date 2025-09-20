use std::os::unix::net::UnixStream;

use mpd::Client;

pub fn get_current(mpd: &mut Client<UnixStream>) -> String {
    let song = mpd
        .currentsong()
        .ok()
        .unwrap_or_default()
        .unwrap_or_default();

    let title = song.title.unwrap_or("Unknown".into());
    let artist = song.artist.unwrap_or("Unknown".into());
    let album = song
        .tags
        .into_iter()
        .filter_map(|(name, val)| {
            if name.to_lowercase().as_str() == "album" {
                Some(val)
            } else {
                None
            }
        })
        .collect::<String>();

    format!("🎵 {title}\n👤 {artist}\n💿 {album}")
}
