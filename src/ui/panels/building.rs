use bevy::prelude::*;

use crate::economy::components::{BuildingPanel, PanelModal};
use crate::ui::panels::building_panel_ui::build_building_panel_ui;
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
        // Close existing panel
        if let Some(e) = panel.root.take() { crate::core::utils::silent_despawn(commands, e); }
        if let Some(e) = panel.overlay.take() { crate::core::utils::silent_despawn(commands, e); }
        if let Some(e) = panel.recipe_selector.take() { crate::core::utils::silent_despawn(commands, e); }
        panel.inspected = None;
        panel.dirty = false;

        // Check if this building has a TOML panel definition
        let def = ctx.building_registry.get(ctx.building_kind);
        if let Some(panel_key) = def.and_then(|d| d.panel.as_deref()) {
            let filename = format!("panel_{}.toml", panel_key);
            if let Some(content) = ctx.mods.load_data(&filename) {
                if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                    let (overlay, root) = ctx.layout_engine.render_panel(
                        commands,
                        &config,
                        ctx.entity,
                        ctx.world,
                    );
                    commands.entity(root).insert(PanelModal);
                    panel.overlay = Some(overlay);
                    panel.root = Some(root);
                    panel.inspected = Some(ctx.entity);
                    panel.dirty = true;
                    return (overlay, root);
                }
            }
        }

        // Fallback: legacy Rust panel
        let kind = ctx.building_kind;
        let show_recipes = def.map_or(false, |d| d.has_recipes);
        let is_farm = kind == "farm";
        let farm_crop_types = def.map(|d| d.crop_types.clone()).unwrap_or_default();
        let modal_size = Vec2::new(800.0, 560.0);

        let overlay = commands.spawn((
            crate::economy::components::PanelOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO, right: Val::ZERO,
                top: Val::ZERO, bottom: Val::ZERO,
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(100),
            Pickable::default(),
        )).id();

        let root = build_building_panel_ui(
            commands, modal_size, ctx.entity, kind,
            show_recipes, is_farm, ctx.resource_registry(),
            ctx.building_registry, farm_crop_types,
        );

        commands.entity(root).insert(PanelModal);
        commands.entity(overlay).add_child(root);
        panel.overlay = Some(overlay);
        panel.root = Some(root);
        panel.inspected = Some(ctx.entity);
        panel.dirty = true;
        (overlay, root)
    }
}
