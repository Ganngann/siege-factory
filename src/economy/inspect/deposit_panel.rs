use crate::core::utils::silent_despawn;
use crate::economy::components::{BuildingPanel, PanelModal, PanelOverlay, ResourceDeposit};
use crate::economy::resource::ResourceRegistry;
use crate::economy::window::{TEXT_GREEN, TEXT_PRIMARY, spawn_window};
use bevy::prelude::*;

use crate::core::game_font::tf;

pub fn spawn_deposit_panel(
    commands: &mut Commands,
    panel: &mut BuildingPanel,
    entity: Entity,
    deposit: &ResourceDeposit,
    resource_registry: &ResourceRegistry,
) {
    let resource_name = resource_registry
        .get_opt(&deposit.resource)
        .map_or(deposit.resource.as_str(), |r| &r.name);

    if let Some(e) = panel.root.take() {
        silent_despawn(commands, e);
    }
    if let Some(e) = panel.overlay.take() {
        silent_despawn(commands, e);
    }
    if let Some(e) = panel.recipe_selector.take() {
        silent_despawn(commands, e);
    }
    panel.inspected = None;
    panel.dirty = false;

    let overlay = commands
        .spawn((
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
        ))
        .id();

    let root = spawn_window(
        commands,
        &format!("Resource Deposit: {}", resource_name),
        super::DEPOSIT_MODAL_WIDTH,
        super::DEPOSIT_MODAL_HEIGHT,
        (1280.0 - super::DEPOSIT_MODAL_WIDTH) / 2.0,
        (720.0 - super::DEPOSIT_MODAL_HEIGHT) / 2.0,
        None,
        |parent| {
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },))
                .with_children(|body| {
                    body.spawn((
                        Text::new(format!("Resource: {}", resource_name)),
                        tf(super::CLOSE_BUTTON_FONT),
                        TextColor(TEXT_PRIMARY),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                    body.spawn((
                        Text::new(format!("Remaining: {}", deposit.amount)),
                        tf(super::CLOSE_BUTTON_FONT),
                        TextColor(TEXT_GREEN),
                    ));
                });
        },
    );
    commands.entity(root).insert(PanelModal);

    commands.entity(overlay).add_child(root);
    panel.overlay = Some(overlay);
    panel.root = Some(root);
    panel.inspected = Some(entity);
    panel.dirty = true;
}
