use bevy::prelude::*;
use std::collections::HashSet;

use crate::economy::components::{
    Active, Assembler, BurnerGenerator, PowerConsumer, PowerPole, PowerProducer, UnbuiltBuilding,
};
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::Inventory;
use crate::map::config::MapConfig;

#[derive(Resource)]
pub struct PowerGrid {
    pub dirty: bool,
    pub utilization_ratio: f32,
}

impl Default for PowerGrid {
    fn default() -> Self {
        Self {
            dirty: true,
            utilization_ratio: 0.0,
        }
    }
}

pub fn detect_power_changes(
    mut grid: ResMut<PowerGrid>,
    added: Query<Entity, Or<(Added<PowerConsumer>, Added<PowerProducer>, Added<PowerPole>)>>,
    mut removals: RemovedComponents<UnbuiltBuilding>,
) {
    if !added.is_empty() || removals.read().count() > 0 {
        grid.dirty = true;
    }
}

fn is_in_range(pos: Vec3, poles: &[(Entity, Vec3, f32)]) -> bool {
    poles
        .iter()
        .any(|(_, pp, range)| pp.distance(pos) <= *range)
}

fn consumer_can_produce(
    entity: Entity,
    spatial_map: &HashSet<Entity>,
    active: Option<&Active>,
    assembler: Option<&Assembler>,
    inventory: Option<&Inventory>,
    recipes: &RecipeRegistry,
) -> bool {
    if !spatial_map.contains(&entity) {
        return false;
    }
    if let Some(a) = active {
        if !a.0 {
            return false;
        }
    }
    if let Some(asm) = assembler {
        if let Some(inv) = inventory {
            let Some(recipe) = recipes.get(&asm.recipe_id) else {
                return false;
            };
            if !recipe
                .input
                .iter()
                .all(|(req_resource, req_amount)| inv.get(req_resource) >= *req_amount)
            {
                return false;
            }
            if inv.capacity > 0 {
                let total_output: u32 = recipe.output.iter().map(|(_, a)| a).sum();
                if inv.total() + total_output > inv.capacity {
                    return false;
                }
            }
        }
    }
    true
}

/// Rebuilds the power grid every frame.
/// Iterates all built producers, burners, poles and consumers to determine:
/// - Which entities are connected (within range of a pole, or all if no poles exist)
/// - Which consumers are actively producing (Active + resources)
/// - The utilization ratio (actual draw / available production)
/// - Which consumers have their power needs satisfied
pub fn rebuild_power_grid(
    mut grid: ResMut<PowerGrid>,
    producers: Query<
        (Entity, &PowerProducer, &Transform),
        (Without<UnbuiltBuilding>, Without<BurnerGenerator>),
    >,
    burners: Query<
        (Entity, &PowerProducer, &Transform),
        (With<BurnerGenerator>, Without<UnbuiltBuilding>),
    >,
    poles: Query<(Entity, &PowerPole, &Transform), Without<UnbuiltBuilding>>,
    mut consumers: Query<
        (
            Entity,
            &Transform,
            &mut PowerConsumer,
            Option<&Active>,
            Option<&Assembler>,
            Option<&Inventory>,
        ),
        Without<UnbuiltBuilding>,
    >,
    recipes: Res<RecipeRegistry>,
    cfg: Res<MapConfig>,
) {
    let pole_data: Vec<(Entity, Vec3, f32)> = poles
        .iter()
        .map(|(e, p, tf)| (e, tf.translation, p.range * cfg.tile_size))
        .collect();
    let has_poles = !pole_data.is_empty();

    let mut connected_consumers: HashSet<Entity> = HashSet::new();

    for (entity, tf, _, _, _, _) in consumers.iter() {
        if !has_poles || is_in_range(tf.translation, &pole_data) {
            connected_consumers.insert(entity);
        }
    }

    let total_available: f32 = producers
        .iter()
        .filter(|(_, _, tf)| !has_poles || is_in_range(tf.translation, &pole_data))
        .map(|(_, p, _)| p.output)
        .sum::<f32>()
        + burners
            .iter()
            .filter(|(_, _, tf)| !has_poles || is_in_range(tf.translation, &pole_data))
            .map(|(_, p, _)| p.output)
            .sum::<f32>();

    let total_actual: f32 = consumers
        .iter()
        .filter(|(entity, _, _, active, assembler, inventory)| {
            consumer_can_produce(
                *entity,
                &connected_consumers,
                *active,
                *assembler,
                *inventory,
                &recipes,
            )
        })
        .map(|(_, _, consumer, _, _, _)| consumer.draw)
        .sum();

    let ratio = if total_available > 0.0 {
        (total_actual / total_available).min(1.0)
    } else {
        0.0
    };
    grid.utilization_ratio = ratio;

    let power_ok = total_available > 0.0 && total_actual <= total_available;

    for (entity, _, mut consumer, active, assembler, inventory) in consumers.iter_mut() {
        let producing = consumer_can_produce(
            entity,
            &connected_consumers,
            active,
            assembler,
            inventory,
            &recipes,
        );
        consumer.satisfied = producing && power_ok;
    }
}

/// Burns fuel proportional to grid utilization.
/// Output is always `base_output` when fuel is present.
/// Fuel consumption rate scales with `utilization_ratio`:
///   ratio=0 → no fuel burned (no one draws power)
///   ratio=0.5 → fuel burns at half speed
///   ratio=1.0 → fuel burns at full speed
pub fn burner_generator_tick(
    time: Res<Time<Fixed>>,
    mut generator_query: Query<
        (&mut BurnerGenerator, &mut Inventory, &mut PowerProducer),
        Without<UnbuiltBuilding>,
    >,
    power_grid: Res<PowerGrid>,
) {
    let ratio = power_grid.utilization_ratio;

    for (mut burner, mut inventory, mut producer) in generator_query.iter_mut() {
        let has_fuel = inventory.resources.iter().any(|(_, &amount)| amount > 0);

        if !has_fuel {
            producer.output = 0.0;
            burner.fuel_burn_timer = 0.0;
            continue;
        }

        // Always output full rated capacity when fuel is available
        producer.output = burner.base_output;

        // Only burn fuel proportional to grid utilization
        if ratio > 0.0 {
            burner.fuel_burn_timer += time.delta_secs() * ratio;
            if burner.fuel_burn_timer >= burner.fuel_burn_interval {
                burner.fuel_burn_timer -= burner.fuel_burn_interval;
                let fuel_key = inventory.resources.keys().next().cloned();
                if let Some(key) = fuel_key {
                    inventory.remove(&key, 1);
                }
            }
        }
    }
}
