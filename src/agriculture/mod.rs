use bevy::prelude::*;

use crate::core::game_state::GameState;
use crate::core::utils::silent_despawn;
use crate::economy::resource::{Inventory, ResourceId};

pub mod components;
pub mod cultivator;
use cultivator::cultivator_ai;

use components::{Crop, CropRegistry, Cultivator, Farm, PendingDeliveries};

pub struct AgriculturePlugin;

impl Plugin for AgriculturePlugin {
    fn build(&self, app: &mut App) {
        let mods = app.world().resource::<crate::core::modding::ModRegistry>().clone();
        app.insert_resource(CropRegistry::load(&mods));
        app.insert_resource(PendingDeliveries::default());
        app.add_systems(
            Update,
            (
                cultivator_ai,
                process_deliveries,
                crop_growth,
                update_crop_visual,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnExit(GameState::Playing), cleanup_crops);
    }
}

fn crop_growth(time: Res<Time>, mut crops: Query<&mut Crop>) {
    for mut crop in crops.iter_mut() {
        crop.timer += time.delta_secs();
    }
}

fn update_crop_visual(mut crops: Query<(&Crop, &mut Sprite)>) {
    for (crop, mut sprite) in crops.iter_mut() {
        let target = if crop.timer >= crop.duration {
            crop.color
        } else {
            dim_color(crop.color, 0.5)
        };
        if sprite.color != target {
            sprite.color = target;
        }
    }
}

fn cleanup_crops(
    mut commands: Commands,
    crops: Query<Entity, With<Crop>>,
    cultivators: Query<Entity, With<Cultivator>>,
) {
    for entity in crops.iter().chain(cultivators.iter()) {
        silent_despawn(&mut commands, entity);
    }
}

fn process_deliveries(
    mut pending: ResMut<PendingDeliveries>,
    mut farm_inventories: Query<&mut Inventory, With<Farm>>,
) {
    for d in pending.0.drain(..) {
        if let Ok(mut inv) = farm_inventories.get_mut(d.farm_entity) {
            inv.add(&ResourceId(d.resource), d.amount);
        }
    }
}

fn dim_color(c: Color, factor: f32) -> Color {
    let srgba = c.to_srgba();
    Color::srgb(
        srgba.red * factor,
        srgba.green * factor,
        srgba.blue * factor,
    )
}
