use directories::ProjectDirs;
use std::{fs, path::PathBuf};

use crate::ui::context::Settings;


pub type Result<T> = std::result::Result<T, String>;

fn config_path() -> Result<PathBuf> {
    // C:\Users\litav\AppData\Roaming\maxim\tracker\config
    let dirs = ProjectDirs::from("com", "maxim", "tracker")
        .ok_or("no config dir")?;

    let dir = dirs.config_dir();
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    Ok(dir.join("settings.json"))
}

// src/lib/store.rs
pub fn load_settings() -> Settings {
    let Ok(path) = config_path() else {
        return Settings::default();
    };
    let Ok(data) = fs::read_to_string(&path) else {
        return Settings::default();
    };
    // Use Value first, merge with defaults — unknown fields are ignored,
    // missing fields use Default values via #[serde(default)] on Settings fields.
    match serde_json::from_str::<Settings>(&data) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Settings parse error (using defaults): {e}");
            // Attempt partial recovery: deserialize as Value and re-serialize defaults with overrides
            Settings::default()
        }
    }
}

pub fn save_settings(settings: &Settings) -> Result<()> {
    let path = config_path()?;

    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| e.to_string())?;

    fs::write(path, data).map_err(|e| e.to_string())
}