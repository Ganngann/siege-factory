use bevy::prelude::*;

use crate::economy::components::{Active, Assembler, PowerConsumer, ProductionCounter};
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
    )>,
) {
    for (mut assembler, mut inventory, active, power, mut counter) in assembler_query.iter_mut() {
        if !active.0 {
            continue;
        }
        if let Some(pc) = power {
            if !pc.satisfied {
                continue;
            }
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

        // Don't produce if inventory is full (capacity > 0)
        if inventory.capacity > 0 {
            let total_output: u32 = recipe.output.iter().map(|(_, a)| a).sum();
            if inventory.total() + total_output > inventory.capacity {
                continue;
            }
        }

        assembler.production_timer += time.delta_secs();
        if assembler.production_timer >= recipe.time_sec {
            for (req_resource, req_amount) in &recipe.input {
                inventory.remove(req_resource, *req_amount);
            }

            for (out_resource, out_amount) in &recipe.output {
                inventory.add(out_resource, *out_amount);
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
