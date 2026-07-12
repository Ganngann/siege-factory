use bevy::prelude::*;

use crate::core::utils::silent_despawn;
use crate::economy::components::BuildingPanel;
use crate::ui::types::PanelType;
use crate::ui::panels::{Panel, PanelSpawnCtx};

pub struct CapsulePanelImpl;

impl Panel for CapsulePanelImpl {
    fn panel_type(&self) -> PanelType { PanelType::Capsule }
    fn panel_name(&self) -> &str { "capsule" }

    fn spawn(
        &self,
        commands: &mut Commands,
        panel: &mut BuildingPanel,
        ctx: &PanelSpawnCtx,
    ) -> (Entity, Entity) {
        if let Some(e) = panel.root.take() { silent_despawn(commands, e); }
        if let Some(e) = panel.overlay.take() { silent_despawn(commands, e); }
        panel.inspected = None;

        let def = ctx.building_registry.get(ctx.building_kind);
        if let Some(panel_key) = def.and_then(|d| d.panel.as_deref()) {
            let filename = format!("panel_{}.toml", panel_key);
            if let Some(content) = ctx.mods.load_data(&filename) {
                match toml::from_str::<toml::Value>(&content) {
                    Ok(config) => {
                        let (overlay, root) = ctx.layout_engine.render_panel(
                            commands, &config, ctx.entity, ctx.data,
                        );
                        panel.overlay = Some(overlay);
                        panel.root = Some(root);
                        panel.inspected = Some(ctx.entity);
                        panel.dirty = true;
                        return (overlay, root);
                    }
                    Err(_) => {}
                }
            }
        }
        let d = commands.spawn(Text::new("")).id();
        (d, d)
    }
}
