use bevy::prelude::*;
use crate::core::game_state::GameState;
use crate::map::components::*;

const TILE_SIZE: f32 = 32.0;
const GRID_WIDTH: u32 = 20;
const GRID_HEIGHT: u32 = 15;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_map);
        app.add_systems(OnExit(GameState::Playing), cleanup_map);
        app.add_systems(Update, handle_tile_click.run_if(in_state(GameState::Playing)));
    }
}

fn setup_map(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            GRID_WIDTH as f32 * TILE_SIZE / 2.0,
            GRID_HEIGHT as f32 * TILE_SIZE / 2.0,
            100.0,
        ),
        ..default()
    });

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let color = if (x + y) % 2 == 0 {
                Color::srgb(0.25, 0.35, 0.25)
            } else {
                Color::srgb(0.18, 0.28, 0.18)
            };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
                    ..default()
                },
                TilePosition { x, y },
            ));
        }
    }
}

fn cleanup_map(mut commands: Commands, entities: Query<Entity, Or<(With<TilePosition>, With<Camera2d>)>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn handle_tile_click(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut tile_query: Query<(&TilePosition, &mut Sprite)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera.single();
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let tile_x = (world_pos.x / TILE_SIZE).floor() as u32;
    let tile_y = (world_pos.y / TILE_SIZE).floor() as u32;

    for (tile_pos, mut sprite) in tile_query.iter_mut() {
        if tile_pos.x == tile_x && tile_pos.y == tile_y {
            sprite.color = Color::srgb(0.5, 0.8, 0.3);
        }
    }
}
