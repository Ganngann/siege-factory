use crate::load_toml;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EnemyDef {
    pub id: String,
    pub name: String,
    pub hp: u32,
    pub speed: f32,
    pub damage: u32,
    pub color: Color,
}

#[derive(Debug, Clone, Resource)]
pub struct EnemyRegistry {
    pub enemies: HashMap<String, EnemyDef>,
}

impl EnemyRegistry {
    pub fn load() -> Self {
        let parsed: EnemiesToml = load_toml!("../../data/enemies.toml", EnemiesToml);
        let mut enemies = HashMap::new();
        for (id, entry) in parsed.enemies {
            let color = entry
                .color
                .as_deref()
                .map(parse_hex_color)
                .unwrap_or(Color::srgb(0.9, 0.2, 0.2));
            enemies.insert(
                id.clone(),
                EnemyDef {
                    id: id.clone(),
                    name: entry.name,
                    hp: entry.hp,
                    speed: entry.speed,
                    damage: entry.damage,
                    color,
                },
            );
        }
        Self { enemies }
    }

    pub fn get(&self, id: &str) -> Option<&EnemyDef> {
        self.enemies.get(id)
    }
}

use crate::core::utils::parse_hex_color;

#[derive(Deserialize)]
struct EnemiesToml {
    enemies: HashMap<String, EnemyEntry>,
}

#[derive(Deserialize)]
struct EnemyEntry {
    name: String,
    hp: u32,
    speed: f32,
    damage: u32,
    color: Option<String>,
}


