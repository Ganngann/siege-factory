use bevy::prelude::*;

use crate::economy::components::BuildingPanel;
use crate::ui::types::PanelType;
use crate::ui::panels::{Panel, PanelSpawnCtx};

// TODO — migrer le contenu de spawn_crafting_panel (crafting.rs) ici
pub struct CraftingPanelImpl;

impl Panel for CraftingPanelImpl {
    fn panel_type(&self) -> PanelType { PanelType::Crafting }
    fn panel_name(&self) -> &str { "crafting" }

    fn spawn(
        &self,
        commands: &mut Commands,
        _panel: &mut BuildingPanel,
        _ctx: &PanelSpawnCtx,
    ) -> (Entity, Entity) {
        let dummy = commands.spawn((
            Text::new("Crafting panel — migration en cours".to_string()),
            crate::core::game_font::tf(12.0),
            TextColor(Color::WHITE),
        )).id();
        (dummy, dummy)
    }
}

