use bevy::prelude::*;

use crate::core::game_font::tf;

use crate::combat::Projectile;
use crate::core::game_state::GameState;
use crate::core::utils::silent_despawn;
use crate::economy::belt::BeltSlots;
use crate::economy::components::Unit;
use crate::economy::components::{
    Builder, Building, Ghost, HpBarChild, PanelModal, Player, ResourceDeposit,
};
use crate::economy::ui::InventoryPanel;
use crate::enemy::components::Enemy as EnemyComponent;
use crate::map::components::ChunkMember;
use crate::map::systems::ChunkMarker;

use super::{SaveManager, SaveRequested, ShowPauseMenu, save_path};

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct LoadButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct QuitButton;

pub fn toggle_pause_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<crate::core::input::KeyBindings>,
    mut show: ResMut<ShowPauseMenu>,
) {
    if bindings.just_pressed("cancel", &keys, &mouse) {
        show.0 = !show.0;
    }
}

pub fn spawn_pause_menu(
    mut commands: Commands,
    show: Res<ShowPauseMenu>,
    panel_query: Query<Entity, With<PauseMenuRoot>>,
) {
    if show.0 && panel_query.is_empty() {
        let _ = commands
            .spawn((
                PauseMenuRoot,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                Pickable::default(),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(24.0)),
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
                        Outline {
                            width: Val::Px(2.0),
                            offset: Val::ZERO,
                            color: Color::srgb(0.4, 0.4, 0.5),
                        },
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new("PAUSED"),
                            tf(28.0),
                            TextColor(Color::srgb(0.8, 0.8, 1.0)),
                            Node {
                                margin: UiRect::bottom(Val::Px(12.0)),
                                ..default()
                            },
                        ));
                        panel
                            .spawn((
                                SaveButton,
                                Button,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(40.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Save Game"),
                                    tf(16.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        panel
                            .spawn((
                                LoadButton,
                                Button,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(40.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Load Game"),
                                    tf(16.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        panel
                            .spawn((
                                ResumeButton,
                                Button,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(40.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Resume"),
                                    tf(16.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        panel
                            .spawn((
                                QuitButton,
                                Button,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(40.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Main Menu"),
                                    tf(16.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
            });
    } else if !show.0 {
        for entity in &panel_query {
            silent_despawn(&mut commands, entity);
        }
    }
}

pub fn resume_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<ResumeButton>)>,
    mut show: ResMut<ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
        }
    }
}

pub fn quit_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show: ResMut<ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            next_state.set(GameState::Menu);
        }
    }
}

pub fn save_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    mut show: ResMut<ShowPauseMenu>,
    mut save_req: ResMut<SaveRequested>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            save_req.0 = true;
        }
    }
}

pub fn load_interaction(
    query: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut save_mgr: ResMut<SaveManager>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show: ResMut<ShowPauseMenu>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            show.0 = false;
            save_mgr.load_requested = Some(save_path().to_string_lossy().to_string());
            next_state.set(GameState::Loading);
        }
    }
}

pub fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuRoot>>) {
    for e in &query {
        silent_despawn(&mut commands, e);
    }
}

pub fn cleanup_world(
    mut commands: Commands,
    belt_slots: Query<&BeltSlots>,
    // SUGGEST: type DespawnQuery = Query<Entity, Or<(With<Building>, ...)>> — envisager un component tag pour cleanup (clippy::type_complexity)
    to_despawn: Query<
        Entity,
        Or<(
            With<Building>,
            With<EnemyComponent>,
            With<Unit>,
            With<ResourceDeposit>,
            With<ChunkMarker>,
            With<ChunkMember>,
            With<Camera2d>,
            With<Ghost>,
            With<HpBarChild>,
            With<crate::economy::components::MenuBarPanel>,
            With<PanelModal>,
            With<InventoryPanel>,
            With<PauseMenuRoot>,
            With<Projectile>,
        )>,
    >,
    // SUGGEST: type PlayerBuilderQuery = Query<Entity, Or<(With<Player>, With<Builder>)>> (clippy::type_complexity)
    player_builder: Query<Entity, Or<(With<Player>, With<Builder>)>>,
) {
    for bs in belt_slots.iter() {
        for sprite_entity in bs.slot_sprites.iter().flatten() {
            silent_despawn(&mut commands, *sprite_entity);
        }
    }
    for e in &to_despawn {
        silent_despawn(&mut commands, e);
    }
    for e in &player_builder {
        silent_despawn(&mut commands, e);
    }
}
