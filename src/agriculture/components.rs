use bevy::prelude::*;
use serde::Deserialize;

#[derive(Resource, Default)]
pub struct PendingDeliveries(pub Vec<PendingDelivery>);

pub struct PendingDelivery {
    pub farm_entity: Entity,
    pub resource: String,
    pub amount: u32,
}

#[derive(Component)]
pub struct Farm {
    pub crop_index: usize,
    pub crop_types: Vec<String>,
}

#[derive(Component)]
pub struct Cultivator {
    pub state: CultivatorState,
    pub carried_resource: Option<String>,
    pub carried_amount: u32,
    pub carry_capacity: u32,
}

#[derive(Clone)]
pub enum CultivatorState {
    Idle,
    MovingToPlant(i32, i32),
    MovingToHarvest(Entity),
    MovingToFarmForSeeds(Entity),
    DeliveringToFarm(Entity),
}

#[derive(Component)]
pub struct Crop {
    pub resource: String,
    pub timer: f32,
    pub duration: f32,
    pub color: Color,
}

#[derive(Deserialize, Clone)]
pub struct CropDef {
    pub name: String,
    pub growth_time: f32,
    pub yield_amount: u32,
    pub color: String,
}

#[derive(Resource)]
pub struct CropRegistry {
    pub crops: std::collections::HashMap<String, CropDef>,
}

impl CropRegistry {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        #[derive(Deserialize)]
        struct CropToml {
            crops: std::collections::HashMap<String, CropDef>,
        }
        let parsed: CropToml = mods.load_toml("crops.toml");
        Self {
            crops: parsed.crops,
        }
    }

    pub fn get(&self, id: &str) -> Option<&CropDef> {
        self.crops.get(id)
    }
}

