use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::core::utils::parse_hex_color;

#[derive(Debug, Clone)]
pub struct BiomeDecoration {
    pub kind: String,
    pub shape: String,
    pub density: f32,
    pub color: Color,
    pub z: f32,
}

#[derive(Debug, Clone)]
pub struct BiomeDef {
    pub id: String,
    pub name: String,
    pub tile_color_even: Color,
    pub tile_color_odd: Color,
    pub decorations: Vec<BiomeDecoration>,
    pub deposits: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Resource)]
pub struct BiomeRegistry {
    pub biomes: HashMap<String, BiomeDef>,
}

impl BiomeRegistry {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let mut biomes = HashMap::new();
        for (_mod_id, parsed) in mods.load_all_toml::<BiomesToml>("biomes.toml") {
            for (id, entry) in parsed.biomes {
                let tile_color_even = entry
                    .tile_color_even
                    .as_deref()
                    .map(parse_hex_color)
                    .unwrap_or(Color::srgb(0.3, 0.5, 0.3));
                let tile_color_odd = entry
                    .tile_color_odd
                    .as_deref()
                    .map(parse_hex_color)
                    .unwrap_or(Color::srgb(0.4, 0.6, 0.4));
                let decorations = entry
                    .decorations
                    .iter()
                    .map(|d| BiomeDecoration {
                        kind: d.kind.clone(),
                        shape: d.shape.clone(),
                        density: d.density,
                        color: d
                            .color
                            .as_deref()
                            .map(parse_hex_color)
                            .unwrap_or(Color::srgb(0.3, 0.5, 0.2)),
                        z: d.z,
                    })
                    .collect();
                biomes.insert(
                    id.clone(),
                    BiomeDef {
                        id,
                        name: entry.name,
                        tile_color_even,
                        tile_color_odd,
                        decorations,
                        deposits: entry
                            .deposits
                            .unwrap_or_default()
                            .into_iter()
                            .collect(),
                    },
                );
            }
        }
        Self { biomes }
    }

    pub fn get(&self, id: &str) -> Option<&BiomeDef> {
        self.biomes.get(id)
    }

    /// Pick a biome deterministically from seed + chunk coords.
    pub fn biome_for_chunk(&self, seed: u64, cx: i32, cy: i32) -> Option<&BiomeDef> {
        if self.biomes.is_empty() {
            return None;
        }
        let hash = crate::map::rng::chunk_hash(seed, cx, cy);
        let idx = hash as usize % self.biomes.len();
        self.biomes.values().nth(idx)
    }
}

#[derive(Deserialize)]
struct BiomesToml {
    #[serde(default)]
    biomes: HashMap<String, BiomeEntry>,
}

#[derive(Deserialize)]
struct BiomeEntry {
    name: String,
    #[serde(default)]
    tile_color_even: Option<String>,
    #[serde(default)]
    tile_color_odd: Option<String>,
    #[serde(default)]
    decorations: Vec<BiomeDecorationEntry>,
    #[serde(default)]
    deposits: Option<HashMap<String, u32>>,
}

#[derive(Deserialize)]
struct BiomeDecorationEntry {
    kind: String,
    shape: String,
    density: f32,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    z: f32,
}
