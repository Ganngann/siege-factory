use bevy::prelude::*;

use crate::economy::components::BuildingPanel;
use crate::ui::panels::{Panel, PanelSpawnCtx};
use crate::ui::types::PanelType;

pub struct BuildingPanelImpl;

impl Panel for BuildingPanelImpl {
    fn panel_type(&self) -> PanelType { PanelType::Building }
    fn panel_name(&self) -> &str { "building" }

    fn spawn(
        &self,
        commands: &mut Commands,
        panel: &mut BuildingPanel,
        ctx: &PanelSpawnCtx,
    ) -> (Entity, Entity) {
        if let Some(e) = panel.root.take() { crate::core::utils::silent_despawn(commands, e); }
        if let Some(e) = panel.overlay.take() { crate::core::utils::silent_despawn(commands, e); }
        panel.inspected = None;
        panel.dirty = false;

        let def = ctx.building_registry.get(ctx.building_kind);
        let panel_key = def.and_then(|d| d.panel.as_deref()).unwrap_or("production");
        let filename = format!("panel_{}.toml", panel_key);
        let config = ctx.mods.load_data(&filename)
            .and_then(|content| toml::from_str::<toml::Value>(&content).ok())
            .unwrap_or_else(|| {
                let mut default = toml::Table::new();
                default.insert("title".into(), toml::Value::String(ctx.building_kind.to_string()));
                default.insert("width".into(), toml::Value::Integer(400));
                default.insert("height".into(), toml::Value::Integer(300));
                toml::Value::Table(default)
            });

        let (overlay, root) = ctx.layout_engine.render_panel(
            commands,
            &config,
            ctx.entity,
            ctx.data,
        );
        panel.overlay = Some(overlay);
        panel.root = Some(root);
        panel.inspected = Some(ctx.entity);
        panel.dirty = true;
        (overlay, root)
    }
}
