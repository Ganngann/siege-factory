use bevy::prelude::*;

use crate::economy::components::{Assembler, Active};
use crate::economy::resource::Inventory;
use crate::economy::recipe::RecipeRegistry;

pub fn assembler_tick(
    time: Res<Time>,
    recipes: Res<RecipeRegistry>,
    mut assembler_query: Query<(&mut Assembler, &mut Inventory, &Active)>,
) {
    for (mut assembler, mut inventory, active) in assembler_query.iter_mut() {
        if !active.0 { continue; }
        let recipe = match recipes.get(&assembler.recipe_id) {
            Some(r) => r,
            None => continue,
        };

        // Check: enough inputs in inventory?
        let can_produce = recipe.input.iter()
            .all(|(req_resource, req_amount)| inventory.get(req_resource) >= *req_amount);

        if !can_produce {
            continue;
        }

        assembler.production_timer += time.delta_secs();
        if assembler.production_timer >= recipe.time_sec {
            // Consume: remove inputs from inventory
            for (req_resource, req_amount) in &recipe.input {
                inventory.remove(req_resource, *req_amount);
            }

            // Produce: add outputs to inventory
            for (out_resource, out_amount) in &recipe.output {
                inventory.add(out_resource, *out_amount);
            }

            assembler.production_timer -= recipe.time_sec;
        }
    }
}
