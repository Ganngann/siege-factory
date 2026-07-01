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
        let toml_str = include_str!("../../data/enemies.toml");
        let parsed: EnemiesToml = toml::from_str(toml_str).expect("failed to parse enemies.toml");
        let mut enemies = HashMap::new();
        for (id, entry) in parsed.enemies {
            let color = entry.color.as_deref()
                .map(parse_hex_color)
                .unwrap_or(Color::srgb(0.9, 0.2, 0.2));
            enemies.insert(id.clone(), EnemyDef {
                id: id.clone(),
                name: entry.name,
                hp: entry.hp,
                speed: entry.speed,
                damage: entry.damage,
                color,
            });
        }
        Self { enemies }
    }

    pub fn get(&self, id: &str) -> Option<&EnemyDef> {
        self.enemies.get(id)
    }
}

fn parse_hex_color(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255) as f32 / 255.0;
    Color::srgb(r, g, b)
}

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
