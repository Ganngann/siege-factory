use bevy::prelude::*;

use crate::economy::components::Player;
use crate::economy::recipe::{RecipeDef, RecipeRegistry};
use crate::economy::resource::Inventory;

// ── Resources ──

#[derive(Resource, Default)]
pub struct CraftingProgress {
    pub active_recipe: Option<String>,
    pub timer: f32,
}

// ── Marker component ──

#[derive(Component)]
pub struct CraftButton {
    pub recipe_id: String,
}

fn can_afford(inv: &Inventory, recipe: &RecipeDef) -> bool {
    recipe
        .input
        .iter()
        .all(|(rid, amount)| inv.get(rid) >= *amount)
}

pub fn craft_button_system(
    // SUGGEST: type CraftButtonQuery = Query<(&Interaction, &CraftButton), (Changed<Interaction>, With<Button>)> (clippy::type_complexity)
    mut interaction_query: Query<
        (&Interaction, &CraftButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut player_query: Query<&mut Inventory, With<Player>>,
    recipe_registry: Res<RecipeRegistry>,
    mut progress: ResMut<CraftingProgress>,
) {
    for (interaction, btn) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if progress.active_recipe.is_some() {
            continue;
        }

        let Ok(mut inv) = player_query.single_mut() else {
            continue;
        };

        let Some(recipe) = recipe_registry.get(&btn.recipe_id) else {
            continue;
        };

        if !can_afford(&inv, recipe) {
            continue;
        }

        for (rid, amount) in &recipe.input {
            inv.remove(rid, *amount);
        }

        progress.active_recipe = Some(btn.recipe_id.clone());
        progress.timer = 0.0;
    }
}

pub fn crafting_tick(
    time: Res<Time>,
    recipe_registry: Res<RecipeRegistry>,
    mut progress: ResMut<CraftingProgress>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    mut toast_queue: ResMut<crate::core::toast::ToastQueue>,
) {
    let Some(ref recipe_id) = progress.active_recipe else {
        return;
    };

    let Some(recipe) = recipe_registry.get(recipe_id) else {
        progress.active_recipe = None;
        return;
    };

    progress.timer += time.delta_secs();
    if progress.timer >= recipe.time_sec {
        let Ok(mut inv) = player_query.single_mut() else {
            return;
        };

        for (rid, amount) in &recipe.output {
            inv.add(rid, *amount);
        }

        let name = recipe
            .output
            .iter()
            .map(|(r, a)| format!("{} x{}", r.display_name(), a))
            .collect::<Vec<_>>()
            .join(", ");
        toast_queue.0.push(format!("Crafted {}", name));

        progress.active_recipe = None;
        progress.timer = 0.0;
    }
}


