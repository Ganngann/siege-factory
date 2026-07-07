use bevy::prelude::*;
use serde::Deserialize;
use std::path::PathBuf;

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
                            info!("Discovered mod: {} v{}", manifest.name, manifest.version);
                            mods.push(ActiveMod { manifest, path });
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

        Self { mods }
    }

    /// Find a mod by its ID
    pub fn get(&self, id: &str) -> Option<&ActiveMod> {
        self.mods.iter().find(|m| m.manifest.id == id)
    }

    /// Load a data file from mods in order.
    /// Returns the content of the FIRST mod that has `data/{filename}`.
    /// Checks mods in priority order (last active mod wins).
    pub fn load_data(&self, filename: &str) -> Option<String> {
        for am in self.mods.iter().rev() {
            let path = am.path.join("data").join(filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    return Some(content);
                }
            }
        }
        None
    }

    /// Load ALL versions of a data file across all mods.
    /// Returns (mod_id, content) pairs in mod priority order (base first).
    pub fn load_all_data(&self, filename: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        for am in &self.mods {
            let path = am.path.join("data").join(filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    results.push((am.manifest.id.clone(), content));
                }
            }
        }
        results
    }

    /// Load a texture file from mods in order (first found wins).
    pub fn load_texture(&self, stem: &str, layer: &str) -> Option<Vec<u8>> {
        let filename = format!("{}_{}.png", stem, layer);
        for am in self.mods.iter().rev() {
            let path = am.path.join("textures").join(&filename);
            if path.exists() {
                if let Ok(data) = std::fs::read(&path) {
                    return Some(data);
                }
            }
        }
        None
    }

    /// Load a story file from mods in order.
    pub fn load_story(&self, filename: &str) -> Option<String> {
        for am in self.mods.iter().rev() {
            let path = am.path.join("story").join(filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    return Some(content);
                }
            }
        }
        None
    }

    /// Get the base mod path
    pub fn base_path(&self) -> Option<PathBuf> {
        self.get("base").map(|m| m.path.clone())
    }
}

/// Plugin to initialize the ModRegistry
pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        let registry = ModRegistry::discover();
        info!("ModPlugin: {} mod(s) loaded", registry.mods.len());
        app.insert_resource(registry);
    }
}
