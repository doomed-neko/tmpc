use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(feature = "local")]
use teloxide::types::FileId;

// Shared state for your bot
pub struct AppState {
    // Maps a short u32 ID to the full MPD file path
    pub track_cache: RwLock<HashMap<u32, String>>,
    #[cfg(feature = "local")]
    pub file_id_cahce: RwLock<HashMap<String, FileId>>,
    pub next_id: AtomicU32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            track_cache: RwLock::new(HashMap::new()),
            #[cfg(feature = "local")]
            file_id_cahce: RwLock::new(HashMap::new()),
            next_id: AtomicU32::new(1),
        }
    }

    pub fn cache_track(&self, uri: String) -> u32 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.track_cache.write().unwrap().insert(id, uri);
        id
    }

    pub fn get_track(&self, id: u32) -> Option<String> {
        self.track_cache.read().unwrap().get(&id).cloned()
    }
}

#[cfg(feature = "local")]
impl AppState {
    pub fn cache_file_id(&self, file_path: String, file_id: FileId) {
        self.file_id_cahce
            .write()
            .unwrap()
            .insert(file_path, file_id);
    }

    // Call this when receiving the callback query
    pub fn get_file_id(&self, song_path: &String) -> Option<FileId> {
        self.file_id_cahce.read().unwrap().get(song_path).cloned()
    }
}
