use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct StatusTier {
    pub tier: usize,
    pub text: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct StatusDef {
    pub id: String,
    pub label: String,
    pub tiers: Vec<StatusTier>,
}

impl StatusDef {
    pub fn status_for_tier(&self, tier: usize) -> &StatusTier {
        let idx = self.tiers.iter().rposition(|t| t.tier <= tier).unwrap_or(0);
        &self.tiers[idx]
    }
}

#[derive(Debug, Clone, Resource)]
pub struct CapsuleStatusRegistry {
    pub statuses: Vec<StatusDef>,
}

impl CapsuleStatusRegistry {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let mut statuses = Vec::new();
        if let Some(content) = mods.load_data("capsule_status.toml") {
            if let Ok(parsed) = toml::from_str::<CapsuleStatusToml>(&content) {
                for entry in parsed.statuses {
                    let tiers: Vec<StatusTier> = entry.tiers.iter().map(|t| StatusTier {
                        tier: t.tier,
                        text: t.text.clone(),
                        color: t.color.clone(),
                    }).collect();
                    statuses.push(StatusDef {
                        id: entry.id,
                        label: entry.label,
                        tiers,
                    });
                }
            }
        }
        Self { statuses }
    }

    pub fn get(&self, id: &str) -> Option<&StatusDef> {
        self.statuses.iter().find(|s| s.id == id)
    }

    pub fn status_text(&self, id: &str, tier: usize) -> String {
        self.get(id).map(|s| s.status_for_tier(tier).text.clone()).unwrap_or_default()
    }

    pub fn status_color(&self, id: &str, tier: usize) -> String {
        self.get(id).map(|s| s.status_for_tier(tier).color.clone()).unwrap_or("#ff3333".into())
    }
}

#[derive(Deserialize)]
struct CapsuleStatusToml {
    statuses: Vec<StatusEntry>,
}

#[derive(Deserialize)]
struct StatusEntry {
    id: String,
    label: String,
    tiers: Vec<TierEntry>,
}

#[derive(Deserialize)]
struct TierEntry {
    tier: usize,
    text: String,
    color: String,
}
