use bevy::prelude::*;
use std::collections::HashSet;

use crate::economy::components::{
    Active, Assembler, BurnerGenerator, PowerConsumer, PowerPole, PowerProducer, ProductionCounter,
    RecipeGenerator, UnbuiltBuilding,
};
use crate::economy::fluid::FluidTank;
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
    // SUGGEST: type AddedQuery = Query<Entity, Or<(Added<PowerConsumer>, Added<PowerProducer>, Added<PowerPole>)>> (clippy::type_complexity)
    added: Query<Entity, Or<(Added<PowerConsumer>, Added<PowerProducer>, Added<PowerPole>)>>,
    mut removals: RemovedComponents<UnbuiltBuilding>,
) {
    if !added.is_empty() || removals.read().count() > 0 {
        grid.dirty = true;
    }
}

pub fn is_in_range(pos: Vec3, poles: &[(Entity, Vec3, f32)]) -> bool {
    poles
        .iter()
        .any(|(_, pp, range)| pp.distance_squared(pos) <= (*range) * (*range))
}

fn has_recipe_resources(
    recipe_id: &str,
    inventory: Option<&Inventory>,
    recipes: &RecipeRegistry,
) -> bool {
    let inv = match inventory {
        Some(i) => i,
        None => return false,
    };
    let Some(recipe) = recipes.get(recipe_id) else {
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
    true
}

pub fn consumer_can_produce(
    entity: Entity,
    spatial_map: &HashSet<Entity>,
    active: Option<&Active>,
    assembler: Option<&Assembler>,
    recipe_gen: Option<&RecipeGenerator>,
    inventory: Option<&Inventory>,
    recipes: &RecipeRegistry,
) -> bool {
    if !spatial_map.contains(&entity) {
        return false;
    }
    if let Some(a) = active
        && !a.0 {
            return false;
        }
    if let Some(asm) = assembler {
        return has_recipe_resources(&asm.recipe_id, inventory, recipes);
    }
    if let Some(rg) = recipe_gen {
        return has_recipe_resources(&rg.recipe_id, inventory, recipes);
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
    // SUGGEST: type ProducerQuery = Query<(Entity, &PowerProducer, &Transform), (Without<UnbuiltBuilding>, Without<BurnerGenerator>)> (clippy::type_complexity)
    producers: Query<
        (Entity, &PowerProducer, &Transform),
        (Without<UnbuiltBuilding>, Without<BurnerGenerator>),
    >,
    // SUGGEST: type BurnerQuery = Query<(Entity, &PowerProducer, &Transform), (With<BurnerGenerator>, Without<UnbuiltBuilding>)> (clippy::type_complexity)
    burners: Query<
        (Entity, &PowerProducer, &Transform),
        (With<BurnerGenerator>, Without<UnbuiltBuilding>),
    >,
    poles: Query<(Entity, &PowerPole, &Transform), Without<UnbuiltBuilding>>,
    // SUGGEST: type ConsumerQuery = Query<(Entity, &Transform, &mut PowerConsumer, Option<&Active>, Option<&Assembler>, Option<&RecipeGenerator>, Option<&Inventory>), Without<UnbuiltBuilding>> (clippy::type_complexity)
    mut consumers: Query<
        (
            Entity,
            &Transform,
            &mut PowerConsumer,
            Option<&Active>,
            Option<&Assembler>,
            Option<&RecipeGenerator>,
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

    for (entity, tf, _, _, _, _, _) in consumers.iter() {
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

    let mut total_actual = 0.0;
    let mut producing_states = Vec::with_capacity(consumers.iter().len());

    for (entity, _, consumer, active, assembler, recipe_gen, inventory) in consumers.iter() {
        let producing = consumer_can_produce(
            entity,
            &connected_consumers,
            active,
            assembler,
            recipe_gen,
            inventory,
            &recipes,
        );
        producing_states.push(producing);
        if producing {
            total_actual += consumer.draw;
        }
    }

    let ratio = if total_available > 0.0 {
        (total_actual / total_available).min(1.0)
    } else {
        0.0
    };
    grid.utilization_ratio = ratio;

    let power_ok = total_available > 0.0 && total_actual <= total_available;

    for ((_, _, mut consumer, _, _, _, _), producing) in consumers.iter_mut().zip(producing_states.into_iter()) {
        consumer.satisfied = producing && power_ok;
    }
}

/// RecipeGenerator: consumes recipe inputs, produces recipe outputs, and
/// sets PowerProducer.output to base_output whenever the recipe is running.
pub fn recipe_generator_tick(
    time: Res<Time<Fixed>>,
    recipes: Res<RecipeRegistry>,
    // SUGGEST: type RgQuery = Query<(&mut RecipeGenerator, &mut Inventory, &mut PowerProducer, &Active, Option<&PowerConsumer>, Option<&mut ProductionCounter>, Option<&mut FluidTank>), Without<UnbuiltBuilding>> (clippy::type_complexity)
    mut rg_query: Query<
        (
            &mut RecipeGenerator,
            &mut Inventory,
            &mut PowerProducer,
            &Active,
            Option<&PowerConsumer>,
            Option<&mut ProductionCounter>,
            Option<&mut FluidTank>,
        ),
        Without<UnbuiltBuilding>,
    >,
) {
    for (mut rg, mut inventory, mut producer, active, power, mut counter, mut tank) in
        rg_query.iter_mut()
    {
        if !active.0 {
            producer.output = 0.0;
            continue;
        }
        if let Some(pc) = power
            && !pc.satisfied {
                producer.output = 0.0;
                continue;
            }

        let recipe = match recipes.get(&rg.recipe_id) {
            Some(r) => r,
            None => {
                producer.output = 0.0;
                continue;
            }
        };

        let can_produce = recipe
            .input
            .iter()
            .all(|(req_resource, req_amount)| inventory.get(req_resource) >= *req_amount);
        if !can_produce {
            producer.output = 0.0;
            continue;
        }

        // Check fluid inputs
        if let Some(ref tank) = tank {
            let has_fluids = recipe
                .fluid_input
                .iter()
                .all(|(res, amt)| tank.get(res) >= *amt);
            if !has_fluids {
                producer.output = 0.0;
                continue;
            }
        } else if !recipe.fluid_input.is_empty() {
            producer.output = 0.0;
            continue;
        }

        // Check item output room
        if inventory.capacity > 0 {
            let total_output: u32 = recipe.output.iter().map(|(_, a)| a).sum();
            if inventory.total() + total_output > inventory.capacity {
                producer.output = 0.0;
                continue;
            }
        }

        // Check fluid output room
        if let Some(ref tank) = tank {
            if tank.capacity > 0.0 && !recipe.fluid_output.is_empty() {
                let total_fluid_out: f32 = recipe.fluid_output.iter().map(|(_, a)| a).sum();
                if tank.total() + total_fluid_out > tank.capacity {
                    producer.output = 0.0;
                    continue;
                }
            }
        }

        rg.production_timer += time.delta_secs();
        if rg.production_timer >= recipe.time_sec {
            for (req_resource, req_amount) in &recipe.input {
                inventory.remove(req_resource, *req_amount);
            }
            // Consume fluid inputs
            if let Some(ref mut tank) = tank {
                for (res, amt) in &recipe.fluid_input {
                    tank.remove(res, *amt);
                }
            }
            for (out_resource, out_amount) in &recipe.output {
                inventory.add(out_resource, *out_amount);
            }
            // Produce fluid outputs
            if let Some(ref mut tank) = tank {
                for (res, amt) in &recipe.fluid_output {
                    tank.add(res, *amt);
                }
            }
            if let Some(ref mut ctr) = counter {
                for (_, amount) in &recipe.output {
                    ctr.0 += amount;
                }
            }
            rg.production_timer -= recipe.time_sec;
        }

        // Output power whenever the recipe is running
        producer.output = rg.base_output;
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
        let has_fuel = inventory.total() > 0;

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
                let fuel_key = inventory.first_resource();
                if let Some(key) = fuel_key {
                    inventory.remove(&key, 1);
                }
            }
        }
    }
}


