use bevy::prelude::*;
use crate::core::game_state::GameState;
use crate::map::components::*;
use crate::map::config::MapConfig;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapConfig::load());
        app.add_systems(OnEnter(GameState::Playing), setup_map);
        app.add_systems(OnExit(GameState::Playing), cleanup_map);
    }
}

fn setup_map(
    mut commands: Commands,
    cfg: Res<MapConfig>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            cfg.width as f32 * cfg.tile_size / 2.0,
            cfg.height as f32 * cfg.tile_size / 2.0,
            100.0,
        ),
        ..default()
    });

    for y in 0..cfg.height {
        for x in 0..cfg.width {
            let color = if (x + y) % 2 == 0 {
                Color::srgb(0.25, 0.35, 0.25)
            } else {
                Color::srgb(0.18, 0.28, 0.18)
            };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(cfg.tile_size, cfg.tile_size)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x as f32 * cfg.tile_size, y as f32 * cfg.tile_size, 0.0),
                    ..default()
                },
                TilePosition { x, y },
            ));
        }
    }
}

#[allow(clippy::type_complexity)]
fn cleanup_map(mut commands: Commands, entities: Query<Entity, Or<(With<TilePosition>, With<Camera2d>)>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
