use crate::core::utils::config_dir;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub keybindings: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            keybindings: HashMap::new(),
        }
    }
}

impl Settings {
    fn path() -> PathBuf {
        config_dir().join("settings.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    if let Ok(settings) = toml::from_str(&content) {
                        return settings;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read settings: {e}");
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    eprintln!("Failed to write settings: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize settings: {e}");
            }
        }
    }
}
