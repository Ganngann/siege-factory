use bevy::prelude::*;
use crate::core::input::KeyBindings;
use crate::core::toast::ToastQueue;
use crate::economy::components::{
    Building, BuildMode, DeconstructMode, OccupiedTiles, Sorter, Assembler, BuildingPopup,
    BuildingPopupRoot, RecipeButton, SorterResourceButton, CloseButton, SorterInvertButton,
};
use crate::economy::belt::BeltSlots;
use crate::economy::recipe::RecipeRegistry;
use crate::economy::resource::{ResourceId, ResourceRegistry, Inventory};
use crate::enemy::components::Health;
use crate::map::config::MapConfig;

fn spawn_popup(
    commands: &mut Commands,
    popup: &mut BuildingPopup,
    entity: Entity,
    building_name: &str,
    building_kind: &str,
    hp: Option<Health>,
    inventory: Option<Inventory>,
    belt: Option<BeltSlots>,
    assembler: Option<Assembler>,
    sorter: Option<Sorter>,
    resource_registry: &ResourceRegistry,
    recipes: &RecipeRegistry,
) {
    let active_color = Color::srgb(0.3, 0.6, 0.3);
    let inactive_color = Color::srgb(0.2, 0.2, 0.25);
    let panel_bg = Color::srgba(0.08, 0.08, 0.12, 0.95);

    let root = commands.spawn((
        BuildingPopupRoot,
        BuildingPopupMarker,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(120.0),
            top: Val::Px(80.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            width: Val::Px(300.0),
            ..default()
        },
        BackgroundColor(panel_bg),
        Outline { width: Val::Px(1.0), offset: Val::ZERO, color: Color::srgb(0.4, 0.4, 0.5) },
    )).id();

    let title_text = format!("=== {} ===", building_name);
    commands.entity(root).with_children(|parent| {
        // Header
        parent.spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                ..default()
            },
        )).with_children(|header| {
            header.spawn((
                Text::new(title_text),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
            ));
            header.spawn((
                CloseButton,
                Button,
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.4, 0.15, 0.15)),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new("✕"),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::WHITE),
                ));
            });
        });

        // Separator
        parent.spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(6.0)),
            ..default()
        }, BackgroundColor(Color::srgb(0.3, 0.3, 0.4))));

        // Recipe section
        if (building_kind == "assembler" || building_kind == "furnace") && assembler.is_some() {
            let asm = assembler.as_ref().unwrap();
            parent.spawn((
                Text::new(format!("Recipe: {}", &asm.recipe_id)),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
            ));

            let mut recipe_ids: Vec<String> = recipes.recipes.keys().cloned().collect();
            recipe_ids.sort();
            for recipe_id in &recipe_ids {
                let is_active = *recipe_id == asm.recipe_id;
                let bg = if is_active { active_color } else { inactive_color };
                parent.spawn((
                    RecipeButton { recipe_id: recipe_id.clone() },
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(26.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        margin: UiRect::vertical(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                )).with_children(|btn| {
                    let prefix = if is_active { "▶ " } else { "  " };
                    btn.spawn((
                        Text::new(format!("{}{}", prefix, recipe_id)),
                        TextFont::from_font_size(13.0),
                        TextColor(if is_active { Color::srgb(0.6, 1.0, 0.6) } else { Color::srgb(0.7, 0.7, 0.8) }),
                    ));
                });
            }
        }

        // Sorter section
        if building_kind == "sorter" && sorter.is_some() {
            let s = sorter.as_ref().unwrap();
            let invert_label = if s.inverted { "Invert: ON" } else { "Invert: OFF" };
            parent.spawn((
                SorterInvertButton,
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(26.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(if s.inverted { Color::srgb(0.4, 0.4, 0.15) } else { inactive_color }),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new(invert_label),
                    TextFont::from_font_size(13.0),
                    TextColor(Color::WHITE),
                ));
            });

            let mut resources: Vec<ResourceId> = resource_registry.resources.keys()
                .map(|k| ResourceId(k.clone())).collect();
            resources.sort_by(|a, b| a.0.cmp(&b.0));
            for res in &resources {
                let is_active = res.0 == s.filter.0;
                let bg = if is_active { active_color } else { inactive_color };
                parent.spawn((
                    SorterResourceButton { resource: res.clone() },
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(24.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        margin: UiRect::vertical(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                )).with_children(|btn| {
                    let prefix = if is_active { "▶ " } else { "  " };
                    btn.spawn((
                        Text::new(format!("{}{}", prefix, res.display_name())),
                        TextFont::from_font_size(12.0),
                        TextColor(if is_active { Color::srgb(0.6, 1.0, 0.6) } else { Color::srgb(0.7, 0.7, 0.8) }),
                    ));
                });
            }
        }

        // Separator
        parent.spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(6.0)),
            ..default()
        }, BackgroundColor(Color::srgb(0.3, 0.3, 0.4))));

        // HP
        if let Some(h) = hp {
            parent.spawn((
                Text::new(format!("HP: {}/{}", h.current, h.max)),
                TextFont::from_font_size(13.0),
                TextColor(Color::WHITE),
                Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() },
            ));
        }

        // Inventory
        if let Some(inv) = inventory {
            if inv.total() > 0 {
                let mut parts = Vec::new();
                for (res_id, amount) in &inv.resources {
                    if let Some(def) = resource_registry.get_opt(&res_id.0) {
                        parts.push(format!("{}: {}", def.name, amount));
                    }
                }
                if !parts.is_empty() {
                    parent.spawn((
                        Text::new(parts.join("  ")),
                        TextFont::from_font_size(12.0),
                        TextColor(Color::srgb(0.8, 0.8, 0.6)),
                        Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() },
                    ));
                }
            }
            if inv.capacity > 0 {
                parent.spawn((
                    Text::new(format!("Capacity: {}/{}", inv.total(), inv.capacity)),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.6, 0.6, 0.8)),
                    Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() },
                ));
            }
        }

        // Belt
        if let Some(ref bs) = belt {
            let occupied = bs.slots.iter().filter(|s| s.is_some()).count();
            if occupied > 0 {
                parent.spawn((
                    Text::new(format!("Items in transit: {}/{}", occupied, bs.slots.len())),
                    TextFont::from_font_size(12.0),
                    TextColor(Color::srgb(0.6, 0.8, 0.6)),
                    Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() },
                ));
            }
        }
    });

    popup.popup_entity = Some(root);
    popup.inspected_entity = Some(entity);
    popup.text_entity = None;
    popup.update_timer = 0.0;
}

/// Detect click on building → mark popup as dirty
pub fn building_inspect_click(
    mut commands: Commands,
    mut popup: ResMut<BuildingPopup>,
    build_mode: Res<BuildMode>,
    deconstruct: Res<DeconstructMode>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
    building_query: Query<(Entity, &OccupiedTiles, &Building)>,
) {
    if build_mode.0.is_some() || deconstruct.0 { return; }
    if !bindings.just_pressed("place", &keys, &buttons) { return; }

    let tile_size = cfg.tile_size;
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor) else { return };

    let tile_x = ((world_pos.x + tile_size / 2.0) / tile_size).floor() as i32;
    let tile_y = ((world_pos.y + tile_size / 2.0) / tile_size).floor() as i32;

    let clicked = building_query.iter().find(|(_, tiles, _)|
        tiles.0.iter().any(|&(x, y)| x == tile_x && y == tile_y)
    );

    if let Some((entity, _, _)) = clicked {
        if popup.inspected_entity == Some(entity) {
            return;
        }
        if let Some(old) = popup.popup_entity.take() {
            commands.entity(old).despawn();
        }
        popup.inspected_entity = Some(entity);
        popup.dirty = true;
    } else if popup.popup_entity.is_some() {
        if let Some(old) = popup.popup_entity.take() {
            commands.entity(old).despawn();
        }
        popup.inspected_entity = None;
    }
}

// ── Recipe button click ──

pub fn recipe_click_system(
    mut popup: ResMut<BuildingPopup>,
    query: Query<(&Interaction, &RecipeButton), Changed<Interaction>>,
    mut assembler_query: Query<&mut Assembler>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = popup.inspected_entity else { continue; };
        if let Ok(mut asm) = assembler_query.get_mut(inspected) {
            asm.recipe_id = btn.recipe_id.clone();
            popup.dirty = true;
        }
    }
}

// ── Sorter resource button click ──

pub fn sorter_resource_click_system(
    mut popup: ResMut<BuildingPopup>,
    query: Query<(&Interaction, &SorterResourceButton), Changed<Interaction>>,
    mut sorter_query: Query<&mut Sorter>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = popup.inspected_entity else { continue; };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.filter = btn.resource.clone();
            toast_queue.0.push(format!("Sorter filter: {}", btn.resource.display_name()));
            popup.dirty = true;
        }
    }
}

// ── Sorter invert button click ──

pub fn sorter_invert_click_system(
    mut popup: ResMut<BuildingPopup>,
    query: Query<&Interaction, (Changed<Interaction>, With<SorterInvertButton>)>,
    mut sorter_query: Query<&mut Sorter>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }
        let Some(inspected) = popup.inspected_entity else { continue; };
        if let Ok(mut sorter) = sorter_query.get_mut(inspected) {
            sorter.inverted = !sorter.inverted;
            let mode = if sorter.inverted { "inverted" } else { "normal" };
            toast_queue.0.push(format!("Sorter: {}", mode));
            popup.dirty = true;
        }
    }
}

// ── Close button ──

pub fn close_button_system(
    mut commands: Commands,
    mut popup: ResMut<BuildingPopup>,
    query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }
        if let Some(old) = popup.popup_entity.take() {
            commands.entity(old).despawn();
        }
        popup.inspected_entity = None;
        return;
    }
}

// ── Escape to close ──

pub fn close_popup_on_escape(
    mut commands: Commands,
    mut popup: ResMut<BuildingPopup>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
) {
    if !keys.just_pressed(bindings.key("cancel")) { return; }
    if let Some(old) = popup.popup_entity.take() {
        commands.entity(old).despawn();
    }
    popup.inspected_entity = None;
}

// ── Spawn/refresh popup with latest data ──

pub fn update_building_popup(
    time: Res<Time>,
    mut popup: ResMut<BuildingPopup>,
    building_query: Query<&Building>,
    assembler_query: Query<&Assembler>,
    sorter_query: Query<&Sorter>,
    health_query: Query<&Health>,
    inventory_query: Query<&Inventory>,
    belt_query: Query<&BeltSlots>,
    resource_registry: Res<ResourceRegistry>,
    recipes: Res<RecipeRegistry>,
    mut commands: Commands,
) {
    let Some(inspected) = popup.inspected_entity else { return };

    // Handle dirty flag (immediate refresh needed after button click)
    if popup.dirty {
        popup.dirty = false;
    } else {
        popup.update_timer += time.delta_secs();
        if popup.update_timer < 0.5 { return; }
    }
    popup.update_timer = 0.0;

    let Ok(building) = building_query.get(inspected) else {
        // Building was destroyed
        if let Some(old) = popup.popup_entity.take() {
            commands.entity(old).despawn();
        }
        popup.inspected_entity = None;
        return;
    };

    if let Some(old) = popup.popup_entity.take() {
        commands.entity(old).despawn();
    }

    let hp = health_query.get(inspected).ok().copied();
    let inventory = inventory_query.get(inspected).ok().cloned();
    let belt = belt_query.get(inspected).ok().cloned();
    let assembler = assembler_query.get(inspected).ok().cloned();
    let sorter = sorter_query.get(inspected).ok().cloned();

    spawn_popup(
        &mut commands, &mut popup, inspected,
        &building.name, &building.kind,
        hp, inventory, belt, assembler, sorter,
        &resource_registry, &recipes,
    );
}

/// Marker for cleanup on state exit
#[derive(Component)]
pub struct BuildingPopupMarker;

pub fn cleanup_popup(mut commands: Commands, query: Query<Entity, With<BuildingPopupMarker>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
