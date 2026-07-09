use bevy::prelude::*;

use crate::core::utils::silent_despawn;
use crate::economy::components::{BuildingPanel, PanelModal, PanelOverlay};
use crate::economy::window::{TEXT_GREEN, TEXT_PRIMARY, spawn_window};
use crate::ui::types::PanelType;
use crate::ui::panels::{Panel, PanelSpawnCtx};

use crate::core::game_font::tf;

pub struct DepositPanelImpl;

impl Panel for DepositPanelImpl {
    fn panel_type(&self) -> PanelType { PanelType::Deposit }
    fn panel_name(&self) -> &str { "deposit" }

    fn spawn(
        &self,
        commands: &mut Commands,
        panel: &mut BuildingPanel,
        ctx: &PanelSpawnCtx,
    ) -> (Entity, Entity) {
        // Close existing panel
        if let Some(e) = panel.root.take() { silent_despawn(commands, e); }
        if let Some(e) = panel.overlay.take() { silent_despawn(commands, e); }
        if let Some(e) = panel.recipe_selector.take() { silent_despawn(commands, e); }
        panel.inspected = None;
        panel.dirty = false;

        let deposit_name = ctx.resource_registry()
            .get_opt(ctx.building_kind)
            .map(|r| r.name.as_str())
            .unwrap_or(ctx.building_kind);

        let amount = 0; // TODO: query ResourceDeposit from entity

        let overlay = commands.spawn((
            PanelOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                top: Val::ZERO,
                bottom: Val::ZERO,
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(100),
            Pickable::default(),
        )).id();

        let root = spawn_window(
            commands,
            &format!("Resource Deposit: {}", deposit_name),
            400.0, 200.0,
            (1280.0 - 400.0) / 2.0,
            (720.0 - 200.0) / 2.0,
            None,
            |parent| {
                parent.spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },)).with_children(|body| {
                    body.spawn((
                        Text::new(format!("Resource: {}", deposit_name)),
                        tf(14.0),
                        TextColor(TEXT_PRIMARY),
                        Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
                    ));
                    body.spawn((
                        Text::new(format!("Remaining: {}", amount)),
                        tf(14.0),
                        TextColor(TEXT_GREEN),
                    ));
                });
            },
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

