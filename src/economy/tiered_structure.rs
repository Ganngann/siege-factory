use crate::core::modding::ModRegistry;
use crate::core::toast::ToastQueue;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{Building, Player};
use crate::economy::discovery::GlobalArchive;
use crate::economy::game_components::CurrentTier;
use crate::economy::resource::Inventory;
use crate::economy::spatial::SpatialRegistry;
use crate::map::components::TilePosition;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: String,
    pub tier: usize,
    pub title: String,
    pub text: String,
}

#[derive(Resource, Default)]
pub struct ProgressionLogRegistry {
    pub logs: Vec<LogEntry>,
    pub unlocked: HashSet<String>,
}

impl ProgressionLogRegistry {
    pub fn load(mods: &ModRegistry) -> Self {
        let mut logs = Vec::new();
        if let Some(content) = mods.load_story("logs.toml") {
            if let Ok(parsed) = toml::from_str::<LogsToml>(&content) {
                for entry in parsed.logs {
                    logs.push(LogEntry {
                        id: entry.id,
                        tier: entry.tier,
                        title: entry.title,
                        text: entry.text,
                    });
                }
            }
        }
        Self {
            logs,
            unlocked: HashSet::new(),
        }
    }

    pub fn unlock(&mut self, id: &str) -> Option<&LogEntry> {
        if self.unlocked.contains(id) {
            return None;
        }
        self.unlocked.insert(id.to_string());
        self.logs.iter().find(|l| l.id == id)
    }
}

#[derive(Deserialize)]
struct LogsToml {
    #[serde(default)]
    logs: Vec<LogEntryToml>,
}

#[derive(Deserialize)]
struct LogEntryToml {
    id: String,
    tier: usize,
    title: String,
    text: String,
}

pub fn structure_interact(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&TilePosition, &mut Inventory), With<Player>>,
    building_q: Query<(Entity, &Building, &TilePosition)>,
    registry: Res<BuildingRegistry>,
    mut tier_q: Query<&mut CurrentTier>,
    mut archive: ResMut<GlobalArchive>,
    mut toasts: ResMut<ToastQueue>,
    spatial: Res<SpatialRegistry>,
    mut progression_logs: ResMut<ProgressionLogRegistry>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok((player_tile, mut player_inv)) = player_q.single_mut() else {
        return;
    };

    let check_tiles = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)];

    for &(dx, dy) in &check_tiles {
        let tx = player_tile.x + dx;
        let ty = player_tile.y + dy;

        let Some(entity) = spatial.at(tx, ty) else {
            continue;
        };

        let Ok((_, building, _)) = building_q.get(entity) else {
            continue;
        };

        let Some(def) = registry.get(&building.kind) else {
            continue;
        };

        if def.tiers.is_empty() {
            continue;
        }

        let current = tier_q.get(entity).map(|t| t.0).unwrap_or(0);

        if current >= def.tiers.len() {
            toasts.0.push(format!("{}: fully upgraded", def.name));
            return;
        }

        let tier_def = &def.tiers[current];

        let can_afford = tier_def
            .required_items
            .iter()
            .all(|(res, amt)| player_inv.get(res) >= *amt);

        if !can_afford {
            let missing: Vec<String> = tier_def
                .required_items
                .iter()
                .filter(|(res, amt)| player_inv.get(res) < *amt)
                .map(|(res, _)| res.display_name())
                .collect();
            toasts
                .0
                .push(format!("{}: need {}", def.name, missing.join(", ")));
            return;
        }

        // Consume items
        for (res, amt) in &tier_def.required_items {
            player_inv.remove(res, *amt);
        }

        // Advance tier
        if let Ok(mut ct) = tier_q.get_mut(entity) {
            ct.0 += 1;
        }

        // Unlock recipes
        for recipe_id in &tier_def.unlock_recipes {
            archive.unlocked_recipes.insert(recipe_id.clone());
        }

        // Unlock progression log
        if let Some(ref log_id) = tier_def.log_id {
            if let Some(entry) = progression_logs.unlock(log_id) {
                toasts
                    .0
                    .push(format!("Log: {} — {}", entry.title, entry.text));
            }
        }

        let new_tier = current + 1;
        toasts
            .0
            .push(format!("{}: upgraded to tier {}", def.name, new_tier));

        return;
    }
}
