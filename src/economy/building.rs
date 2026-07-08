use crate::core::utils::parse_hex_color;
use crate::economy::components::{BurnerGenerator, PowerConsumer, PowerPole, PowerProducer};
use crate::economy::game_components::BeltVariant;
use crate::economy::resource::{Cost, ResourceId};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CombatStats {
    pub damage: u32,
    pub range: f32,
    pub fire_rate_sec: f32,
    pub projectile_speed: f32,
}

#[derive(Debug, Clone)]
pub struct BeltProperties {
    pub slots: u32,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub struct ProductionDef {
    pub resource: ResourceId,
    pub interval_sec: f32,
}

#[derive(Debug, Clone)]
pub struct BuildingDef {
    pub id: String,
    pub name: String,
    pub cost: Vec<Cost>,
    pub hp: u32,
    pub tile_size: (u32, u32),
    pub color: Color,
    pub visual: String,
    pub texture_stem: String,
    pub requires_deposit: bool,
    pub infinite_extraction: bool,
    pub combat: Option<CombatStats>,
    pub belt: Option<BeltProperties>,
    pub production: Option<ProductionDef>,
    pub production_interval: Option<f32>,
    pub can_deconstruct: bool,
    pub refund_ratio: f32,
    pub repair_cost_ratio: f32,
    pub inventory_capacity: u32,
    pub compactor_ratio: u32,
    pub compactor_interval: f32,
    pub hidden: bool,
    pub drag_placement: bool,
    pub default_recipe: Option<String>,
    pub default_filter: Option<String>,
    pub crop_types: Vec<String>,
    pub recipe_categories: Vec<String>,
    pub power_consumption: f32,
    pub power_generation: f32,
    pub power_pole_range: f32,
    pub fuel_burn_interval: f32,
    pub requires_discovery: Option<String>,
    pub level: u32,
    pub upgrades_from: Option<String>,
    pub upgrades_to: Option<String>,
    pub belt_variant: BeltVariant,
    pub powered: bool,
    pub station: bool,
    pub tiers: Vec<TierDef>,
}

#[derive(Debug, Clone)]
pub struct TierDef {
    pub required_items: Vec<(ResourceId, u32)>,
    pub unlock_recipes: Vec<String>,
    pub log_id: Option<String>,
    pub texture: String,
}

/// Common power component attachment logic.
pub fn attach_power_components(entity: &mut EntityCommands, def: &BuildingDef) {
    if def.power_consumption > 0.0 {
        entity.insert(PowerConsumer {
            draw: def.power_consumption,
            satisfied: false,
        });
    }
    if def.power_generation > 0.0 {
        entity.insert(PowerProducer {
            output: def.power_generation,
        });
    }
    // BurnerGenerator is only for simple fuel-burning generators.
    // Recipe-based generators (which have a default_recipe) use RecipeGenerator instead.
    if def.fuel_burn_interval > 0.0 && def.default_recipe.is_none() {
        entity.insert(BurnerGenerator {
            fuel_burn_timer: 0.0,
            fuel_burn_interval: def.fuel_burn_interval,
            base_output: def.power_generation,
        });
    }
    if def.power_pole_range > 0.0 {
        entity.insert(PowerPole {
            range: def.power_pole_range,
        });
    }
}

#[derive(Debug, Clone, Resource)]
pub struct DefaultSettings {
    pub can_deconstruct: bool,
    pub refund_ratio: f32,
    pub repair_cost_ratio: f32,
    pub inventory_capacity: u32,
    pub default_projectile_speed: f32,
}

impl DefaultSettings {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let parsed: BuildingsToml = mods.load_toml("buildings.toml");
        Self {
            can_deconstruct: parsed.defaults.can_deconstruct,
            refund_ratio: parsed.defaults.refund_ratio,
            repair_cost_ratio: parsed.defaults.repair_cost_ratio,
            inventory_capacity: parsed.defaults.inventory_capacity,
            default_projectile_speed: parsed.defaults.default_projectile_speed,
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct BuildingRegistry {
    pub buildings: Vec<BuildingDef>,
}

impl BuildingRegistry {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let defaults = &DefaultSettings::load(mods);
        let mut all_buildings: HashMap<String, BuildingEntry> = HashMap::new();
        for (_mod_id, parsed) in mods.load_all_toml::<BuildingsToml>("buildings.toml") {
            for (id, entry) in parsed.buildings {
                all_buildings.insert(id, entry);
            }
        }

        let mut buildings: Vec<BuildingDef> = all_buildings
            .into_iter()
            .map(|(id, entry)| {
                let mut cost = Vec::new();
                for (res_key, amount) in entry.cost {
                    cost.push(Cost {
                        resource: ResourceId::new(res_key),
                        amount,
                    });
                }
                let color = entry
                    .color
                    .as_deref()
                    .map(parse_hex_color)
                    .unwrap_or(Color::srgb(0.5, 0.5, 0.5));
                let visual = entry.visual.unwrap_or_else(|| "square".to_string());
                let texture_stem = entry.texture_stem.unwrap_or_else(|| id.clone());
                let requires_deposit = entry.requires_deposit;
                let infinite_extraction = entry.infinite_extraction;
                let combat = entry.combat.map(|c| CombatStats {
                    damage: c.damage,
                    range: c.range * c.range,
                    fire_rate_sec: c.fire_rate_sec,
                    projectile_speed: c.projectile_speed,
                });
                let production = entry.production.map(|p| ProductionDef {
                    resource: ResourceId::new(&p.resource),
                    interval_sec: p.interval_sec,
                });

                let recipe_categories = entry.recipe_categories.clone();

                let belt = match entry.belt {
                    Some(b) => Some(BeltProperties {
                        slots: b.slots,
                        speed: b.speed,
                    }),
                    None => {
                        let slots = entry.slots.unwrap_or(2);
                        let speed = entry.speed.unwrap_or(2.0);
                        (entry.slots.is_some() || entry.speed.is_some())
                            .then_some(BeltProperties { slots, speed })
                    }
                };

                let belt_variant = entry
                    .belt_variant
                    .as_deref()
                    .map(parse_belt_variant)
                    .unwrap_or_default();

                let tiers = entry
                    .tiers
                    .iter()
                    .map(|t| TierDef {
                        required_items: t
                            .required_items
                            .iter()
                            .map(|(k, v)| (ResourceId::new(k), *v))
                            .collect(),
                        unlock_recipes: t.unlock_recipes.clone(),
                        log_id: t.log_id.clone(),
                        texture: t.texture.clone(),
                    })
                    .collect();

                BuildingDef {
                    id: id.clone(),
                    name: entry.name,
                    cost,
                    hp: entry.hp,
                    tile_size: (entry.tile_size.w, entry.tile_size.h),
                    color,
                    visual,
                    texture_stem,
                    requires_deposit,
                    infinite_extraction,
                    combat,
                    belt,
                    production,
                    production_interval: entry.production_interval,
                    can_deconstruct: entry.can_deconstruct.unwrap_or(defaults.can_deconstruct),
                    refund_ratio: entry.refund_ratio.unwrap_or(defaults.refund_ratio),
                    repair_cost_ratio: entry
                        .repair_cost_ratio
                        .unwrap_or(defaults.repair_cost_ratio),
                    inventory_capacity: entry
                        .inventory_capacity
                        .unwrap_or(defaults.inventory_capacity),
                    compactor_ratio: entry.compactor_ratio,
                    compactor_interval: entry.compactor_interval,
                    hidden: entry.hidden,
                    drag_placement: entry.drag_placement,
                    default_recipe: entry.default_recipe.clone(),
                    default_filter: entry.default_filter.clone(),
                    crop_types: entry.crop_types.clone(),
                    recipe_categories,
                    power_consumption: entry.power_consumption,
                    power_generation: entry.power_generation,
                    power_pole_range: entry.power_pole_range,
                    fuel_burn_interval: entry.fuel_burn_interval,
                    requires_discovery: entry.requires_discovery.clone(),
                    level: entry.level,
                    upgrades_from: entry.upgrades_from.clone(),
                    upgrades_to: None,
                    belt_variant,
                    powered: entry.powered,
                    station: entry.station,
                    tiers,
                }
            })
            .collect();

        // Compute upgrades_to: for each building, check if any other building upgrades from it
        let mut upgrades_map: HashMap<String, String> = HashMap::new();
        for b in &buildings {
            if let Some(ref from) = b.upgrades_from {
                upgrades_map.insert(from.clone(), b.id.clone());
            }
        }
        for b in &mut buildings {
            b.upgrades_to = upgrades_map.get(b.id.as_str()).cloned();
        }

        Self { buildings }
    }

    pub fn get(&self, id: &str) -> Option<&BuildingDef> {
        self.buildings.iter().find(|b| b.id == id)
    }
}

#[derive(Deserialize)]
struct BuildingsToml {
    defaults: DefaultsEntry,
    #[serde(default)]
    buildings: HashMap<String, BuildingEntry>,
}

#[derive(Deserialize)]
struct DefaultsEntry {
    can_deconstruct: bool,
    refund_ratio: f32,
    repair_cost_ratio: f32,
    inventory_capacity: u32,
    #[serde(default = "default_projectile_speed")]
    default_projectile_speed: f32,
}

#[derive(Deserialize)]
struct BuildingEntry {
    name: String,
    #[serde(default)]
    cost: HashMap<String, u32>,
    hp: u32,
    tile_size: TileSize,
    color: Option<String>,
    #[serde(default)]
    visual: Option<String>,
    #[serde(default)]
    texture_stem: Option<String>,
    #[serde(default)]
    requires_deposit: bool,
    #[serde(default)]
    infinite_extraction: bool,
    #[serde(default)]
    production: Option<ProductionEntry>,
    #[serde(default)]
    production_interval: Option<f32>,
    #[serde(default)]
    combat: Option<CombatEntry>,
    #[serde(default)]
    belt: Option<BeltEntry>,
    #[serde(default)]
    slots: Option<u32>,
    #[serde(default)]
    speed: Option<f32>,
    #[serde(default)]
    can_deconstruct: Option<bool>,
    #[serde(default)]
    refund_ratio: Option<f32>,
    #[serde(default)]
    repair_cost_ratio: Option<f32>,
    #[serde(default)]
    inventory_capacity: Option<u32>,
    #[serde(default = "default_compactor_ratio")]
    compactor_ratio: u32,
    #[serde(default = "default_compactor_interval")]
    compactor_interval: f32,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    drag_placement: bool,
    #[serde(default)]
    default_recipe: Option<String>,
    #[serde(default)]
    default_filter: Option<String>,
    #[serde(default)]
    crop_types: Vec<String>,
    #[serde(default)]
    recipe_categories: Vec<String>,
    #[serde(default)]
    power_consumption: f32,
    #[serde(default)]
    power_generation: f32,
    #[serde(default)]
    power_pole_range: f32,
    #[serde(default)]
    fuel_burn_interval: f32,
    #[serde(default)]
    requires_discovery: Option<String>,
    #[serde(default = "default_level")]
    level: u32,
    #[serde(default)]
    upgrades_from: Option<String>,
    #[serde(default)]
    belt_variant: Option<String>,
    #[serde(default = "default_true")]
    powered: bool,
    #[serde(default)]
    station: bool,
    #[serde(default)]
    tiers: Vec<TierEntry>,
}

#[derive(Deserialize)]
struct TierEntry {
    #[serde(default)]
    required_items: HashMap<String, u32>,
    #[serde(default)]
    unlock_recipes: Vec<String>,
    #[serde(default)]
    log_id: Option<String>,
    #[serde(default)]
    texture: String,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct TileSize {
    w: u32,
    h: u32,
}

#[derive(Deserialize)]
struct CombatEntry {
    damage: u32,
    range: f32,
    fire_rate_sec: f32,
    #[serde(default = "default_projectile_speed")]
    projectile_speed: f32,
}

fn default_projectile_speed() -> f32 {
    300.0
}

fn default_level() -> u32 {
    1
}

fn default_compactor_ratio() -> u32 {
    4
}

fn default_compactor_interval() -> f32 {
    2.0
}

fn parse_belt_variant(s: &str) -> BeltVariant {
    match s.to_lowercase().as_str() {
        "underground" => BeltVariant::Underground,
        "aerial" => BeltVariant::Aerial,
        "curved" => BeltVariant::Curved,
        _ => BeltVariant::Normal,
    }
}

#[derive(Deserialize)]
struct BeltEntry {
    slots: u32,
    speed: f32,
}

#[derive(Deserialize)]
struct ProductionEntry {
    resource: String,
    interval_sec: f32,
}



