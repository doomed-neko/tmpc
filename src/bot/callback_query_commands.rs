use core::fmt;

pub enum CallbackCmd {
    Play,
    Pause,
    Stop,
    PlayPause,
    Previous,
    Next,
    ToggleRepeat,
    ToggleRandom,
    ToggleSingle,
    ToggleConsume,
    Clear,
    Add(u32),
    ShuffleQueue,
    Unknown,
}

impl From<String> for CallbackCmd {
    fn from(value: String) -> Self {
        let lower = value.to_lowercase();

        // Handle commands that might contain parameters separated by a colon, e.g., "search:my_query"
        if lower.starts_with("search") {
            let parts: Vec<&str> = value.splitn(2, ':').collect();
            let query = parts.get(1).map_or("", |s| s.trim()).parse().unwrap();
            return Self::Add(query);
        }

        match lower.as_str() {
            "play" => Self::Play,
            "pause" => Self::Pause,
            "stop" => Self::Stop,
            "playpause" | "toggle_playback" => Self::PlayPause,
            "prev" | "previous" => Self::Previous,
            "next" => Self::Next,
            "repeat" | "toggle_repeat" => Self::ToggleRepeat,
            "random" | "toggle_random" => Self::ToggleRandom,
            "single" | "toggle_single" => Self::ToggleSingle,
            "consume" | "toggle_consume" => Self::ToggleConsume,
            "clear" => Self::Clear,
            "shuffle" | "shuffle_queue" => Self::ShuffleQueue,
            _ => Self::Unknown,
        }
    }
}

// Implementing From<CallbackCmd> for String
impl From<CallbackCmd> for String {
    fn from(cmd: CallbackCmd) -> Self {
        match cmd {
            CallbackCmd::Play => "play".to_string(),
            CallbackCmd::Pause => "pause".to_string(),
            CallbackCmd::Stop => "stop".to_string(),
            CallbackCmd::PlayPause => "playpause".to_string(),
            CallbackCmd::Previous => "previous".to_string(),
            CallbackCmd::Next => "next".to_string(),
            CallbackCmd::ToggleRepeat => "toggle_repeat".to_string(),
            CallbackCmd::ToggleRandom => "toggle_random".to_string(),
            CallbackCmd::ToggleSingle => "toggle_single".to_string(),
            CallbackCmd::ToggleConsume => "toggle_consume".to_string(),
            CallbackCmd::Clear => "clear".to_string(),
            CallbackCmd::Add(query) => format!("search:{}", query),
            CallbackCmd::ShuffleQueue => "shuffle_queue".to_string(),
            CallbackCmd::Unknown => "unknown".to_string(),
        }
    }
}

impl fmt::Display for CallbackCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Play => write!(f, "play"),
            Self::Pause => write!(f, "pause"),
            Self::Stop => write!(f, "stop"),
            Self::PlayPause => write!(f, "playpause"),
            Self::Previous => write!(f, "previous"),
            Self::Next => write!(f, "next"),
            Self::ToggleRepeat => write!(f, "toggle_repeat"),
            Self::ToggleRandom => write!(f, "toggle_random"),
            Self::ToggleSingle => write!(f, "toggle_single"),
            Self::ToggleConsume => write!(f, "toggle_consume"),
            Self::Clear => write!(f, "clear"),
            Self::Add(query) => write!(f, "search:{}", query),
            Self::ShuffleQueue => write!(f, "shuffle_queue"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
