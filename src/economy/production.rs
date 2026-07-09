use bevy::prelude::*;

use crate::economy::components::{Active, Assembler, PowerConsumer, ProductionCounter};
use crate::economy::fluid::FluidTank;
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::Inventory;

pub fn assembler_tick(
    time: Res<Time<Fixed>>,
    recipes: Res<RecipeRegistry>,
    mut assembler_query: Query<(
        &mut Assembler,
        &mut Inventory,
        &Active,
        Option<&PowerConsumer>,
        Option<&mut ProductionCounter>,
        Option<&mut FluidTank>,
    )>,
) {
    for (mut assembler, mut inventory, active, power, mut counter, mut tank) in
        assembler_query.iter_mut()
    {
        if !active.0 {
            continue;
        }
        if let Some(pc) = power
            && !pc.satisfied {
                continue;
            }
        let recipe = match recipes.get(&assembler.recipe_id) {
            Some(r) => r,
            None => continue,
        };

        let can_produce = recipe
            .input
            .iter()
            .all(|(req_resource, req_amount)| inventory.get(req_resource) >= *req_amount);

        if !can_produce {
            continue;
        }

        // Check fluid inputs
        if let Some(ref tank) = tank {
            let has_fluids = recipe
                .fluid_input
                .iter()
                .all(|(res, amt)| tank.get(res) >= *amt);
            if !has_fluids {
                continue;
            }
        } else if !recipe.fluid_input.is_empty() {
            continue;
        }

        // Check output room (items)
        if inventory.capacity > 0 {
            let total_output: u32 = recipe.output.iter().map(|(_, a)| a).sum();
            if inventory.total() + total_output > inventory.capacity {
                continue;
            }
        }

        // Check fluid output room
        if let Some(ref tank) = tank {
            if tank.capacity > 0.0 && !recipe.fluid_output.is_empty() {
                let total_fluid_out: f32 = recipe.fluid_output.iter().map(|(_, a)| a).sum();
                if tank.total() + total_fluid_out > tank.capacity {
                    continue;
                }
            }
        }

        assembler.production_timer += time.delta_secs();
        if assembler.production_timer >= recipe.time_sec {
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

            assembler.production_timer -= recipe.time_sec;
        }
    }
}
