use crate::economy::components::{Assembler, BuildingPanel};
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct RecipeRow {
    pub recipe_id: String,
}

pub fn recipe_row_click_system(
    mut panel: ResMut<BuildingPanel>,
    q: Query<(&Interaction, &RecipeRow), Changed<Interaction>>,
    mut asm_q: Query<&mut Assembler>,
) {
    for (interaction, row) in &q {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = panel.inspected else { continue; };
        if let Ok(mut asm) = asm_q.get_mut(inspected) {
            asm.recipe_id = row.recipe_id.clone();
            panel.dirty = true;
        }
    }
}
