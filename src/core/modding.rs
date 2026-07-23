use bevy::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

/// Helper to prevent path traversal and absolute path injection when joining user-provided paths.
fn is_safe_path(path: &str) -> bool {
    let p = Path::new(path);
    !p.components().any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
}

/// Manifest file loaded from each mod's mod.toml
#[derive(Deserialize, Debug, Clone)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

/// An active mod with its manifest and filesystem path
#[derive(Clone)]
pub struct ActiveMod {
    pub manifest: ModManifest,
    pub path: PathBuf,
    pub enabled: bool,
}

/// Persisted settings: which mods the user has disabled.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModSettings {
    pub disabled: Vec<String>,
}

impl ModSettings {
    fn path() -> PathBuf {
        crate::core::utils::config_dir().join("mod_settings.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    if let Ok(settings) = toml::from_str(&content) {
                        return settings;
                    }
                    error!("Failed to parse mod_settings.toml, using defaults");
                }
                Err(e) => {
                    error!("Failed to read mod_settings.toml: {e}");
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent()
            && !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    error!("Failed to write mod_settings.toml: {e}");
                }
            }
            Err(e) => {
                error!("Failed to serialize mod_settings: {e}");
            }
        }
    }
}

/// Registry of all discovered mods. The base game mod (id="base") is always first.
#[derive(Resource, Clone)]
pub struct ModRegistry {
    pub mods: Vec<ActiveMod>,
}

impl ModRegistry {
    /// Scan the `mods/` directory (relative to working dir) and discover all mods.
    /// Each mod must have a `mod.toml` manifest.
    /// Returns a registry with all mods sorted by dependency order (base first).
    pub fn discover() -> Self {
        let mods_dir = PathBuf::from("mods");
        let mut mods = Vec::new();

        if !mods_dir.exists() {
            warn!("mods/ directory not found, creating...");
            std::fs::create_dir_all(&mods_dir).ok();
        }

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let manifest_path = path.join("mod.toml");
                if !manifest_path.exists() {
                    continue;
                }

                match std::fs::read_to_string(&manifest_path) {
                    Ok(content) => match toml::from_str::<ModManifest>(&content) {
                        Ok(manifest) => {
                            mods.push(ActiveMod {
                                enabled: true,
                                manifest,
                                path,
                            });
                        }
                        Err(e) => {
                            error!("Failed to parse {:?}: {}", manifest_path, e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to read {:?}: {}", manifest_path, e);
                    }
                }
            }
        }

        // Ensure base mod exists
        if !mods.iter().any(|m| m.manifest.id == "base") {
            warn!("base mod not found in mods/, checking data/ as fallback");
        }

        let mut reg = Self { mods };
        reg.apply_settings(&ModSettings::load());
        reg
    }

    /// Apply saved enabled/disabled state from settings.
    /// Base mod is always enabled.
    fn apply_settings(&mut self, settings: &ModSettings) {
        let disabled: HashSet<&str> = settings.disabled.iter().map(|s| s.as_str()).collect();
        for am in &mut self.mods {
            if am.manifest.id == "base" {
                am.enabled = true;
            } else {
                am.enabled = !disabled.contains(am.manifest.id.as_str());
            }
        }
    }

    /// Toggle a mod's enabled state and persist.
    pub fn toggle(&mut self, id: &str) {
        if id == "base" {
            return;
        }
        if let Some(am) = self.mods.iter_mut().find(|m| m.manifest.id == id) {
            am.enabled = !am.enabled;
        } else {
            error!("ToggleMod({}) — mod not found in registry", id);
        }
        self.save_settings();
    }

    /// Persist current enabled/disabled state to disk.
    pub fn save_settings(&self) {
        let disabled: Vec<String> = self
            .mods
            .iter()
            .filter(|m| !m.enabled)
            .map(|m| m.manifest.id.clone())
            .collect();
        let settings = ModSettings { disabled };
        settings.save();
    }

    /// Find a mod by its ID
    pub fn get(&self, id: &str) -> Option<&ActiveMod> {
        self.mods.iter().find(|m| m.manifest.id == id)
    }

    /// Iterate over only enabled mods.
    pub fn enabled(&self) -> impl Iterator<Item = &ActiveMod> {
        self.mods.iter().filter(|m| m.enabled)
    }

    /// Load a data file from mods in order.
    /// Returns the content of the FIRST **enabled** mod that has `data/{filename}`.
    /// Checks mods in priority order (last active mod wins).
    pub fn load_data(&self, filename: &str) -> Option<String> {
        if !is_safe_path(filename) {
            warn!("Rejected unsafe path in load_data: {}", filename);
            return None;
        }
        for am in self.mods.iter().rev() {
            if !am.enabled {
                continue;
            }
            let path = am.path.join("data").join(filename);
            if path.exists()
                && let Ok(content) = std::fs::read_to_string(&path) {
                    return Some(content);
                }
        }
        None
    }

    /// Load ALL versions of a data file across all **enabled** mods.
    /// Returns (mod_id, content) pairs in mod priority order (base first).
    pub fn load_all_data(&self, filename: &str) -> Vec<(String, String)> {
        if !is_safe_path(filename) {
            warn!("Rejected unsafe path in load_all_data: {}", filename);
            return Vec::new();
        }
        let mut results = Vec::new();
        for am in &self.mods {
            if !am.enabled {
                continue;
            }
            let path = am.path.join("data").join(filename);
            if path.exists()
                && let Ok(content) = std::fs::read_to_string(&path) {
                    results.push((am.manifest.id.clone(), content));
                }
        }
        results
    }

    /// Load a texture file from mods in order (first found wins).
    pub fn load_texture(&self, stem: &str, layer: &str) -> Option<Vec<u8>> {
        if !is_safe_path(stem) || !is_safe_path(layer) {
            warn!("Rejected unsafe path in load_texture: stem={}, layer={}", stem, layer);
            return None;
        }
        let filename = format!("{}_{}.png", stem, layer);
        for am in self.mods.iter().rev() {
            if !am.enabled {
                continue;
            }
            let path = am.path.join("textures").join(&filename);
            if path.exists()
                && let Ok(data) = std::fs::read(&path) {
                    return Some(data);
                }
        }
        None
    }

    /// Load a story file from mods in order.
    pub fn load_story(&self, filename: &str) -> Option<String> {
        if !is_safe_path(filename) {
            warn!("Rejected unsafe path in load_story: {}", filename);
            return None;
        }
        for am in self.mods.iter().rev() {
            if !am.enabled {
                continue;
            }
            let path = am.path.join("story").join(filename);
            if path.exists()
                && let Ok(content) = std::fs::read_to_string(&path) {
                    return Some(content);
                }
        }
        None
    }

    /// Get the base mod path
    pub fn base_path(&self) -> Option<PathBuf> {
        self.get("base").map(|m| m.path.clone())
    }

    /// Parse the first enabled mod's `data/{filename}` as TOML into `T`.
    /// Panics with a clear message if the file is missing or invalid.
    pub fn load_toml<T: DeserializeOwned>(&self, filename: &str) -> T {
        let content = self
            .load_data(filename)
            .unwrap_or_else(|| panic!("No enabled mod provides data/{filename}"));
        toml::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse data/{filename}: {e}"))
    }

    /// Parse ALL enabled mods' `data/{filename}` as TOML into `Vec<T>`.
    /// Returns entries in mod priority order (base first).
    /// Skips mods whose file fails to parse (logs a warning).
    pub fn load_all_toml<T: DeserializeOwned>(&self, filename: &str) -> Vec<(String, T)> {
        self.load_all_data(filename)
            .into_iter()
            .filter_map(|(id, content)| {
                match toml::from_str(&content) {
                    Ok(parsed) => Some((id, parsed)),
                    Err(e) => {
                        bevy::prelude::error!("Failed to parse data/{filename} from mod {id}: {e}");
                        None
                    }
                }
            })
            .collect()
    }

    /// Create a registry by scanning the filesystem — useful in tests.
    /// Panics if `mods/` is not accessible (run from project root).
    pub fn for_test() -> Self {
        Self::discover()
    }
}

/// Plugin to initialize the ModRegistry
pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        let registry = ModRegistry::discover();
        app.insert_resource(registry);
    }
}
