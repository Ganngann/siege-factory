use bevy::prelude::*;
use bevy_pancam::PanCam;
use crate::core::game_state::GameState;
use crate::map::components::*;
use crate::map::config::MapConfig;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapConfig::load());
        app.insert_resource(HoveredTile::default());
        app.add_systems(OnEnter(GameState::Playing), setup_map);
        app.add_systems(OnExit(GameState::Playing), cleanup_map);
        app.add_systems(Update, update_hovered_tile.run_if(in_state(GameState::Playing)));
    }
}

fn setup_map(mut commands: Commands, cfg: Res<MapConfig>) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(
            cfg.width as f32 * cfg.tile_size / 2.0,
            cfg.height as f32 * cfg.tile_size / 2.0,
            100.0,
        ),
        PanCam {
            speed: 500.0,
            min_scale: 0.3,
            max_scale: 3.0,
            min_x: 0.0,
            max_x: cfg.width as f32 * cfg.tile_size,
            min_y: 0.0,
            max_y: cfg.height as f32 * cfg.tile_size,
            ..default()
        },
    ));

    for y in 0..cfg.height {
        for x in 0..cfg.width {
            let color = if (x + y) % 2 == 0 {
                Color::srgb(0.25, 0.35, 0.25)
            } else {
                Color::srgb(0.18, 0.28, 0.18)
            };

            commands.spawn((
                Sprite::from_color(color, Vec2::new(cfg.tile_size, cfg.tile_size)),
                Transform::from_xyz(x as f32 * cfg.tile_size, y as f32 * cfg.tile_size, 0.0),
                TilePosition { x, y },
            ));
        }
    }
}

fn update_hovered_tile(
    mut hovered: ResMut<HoveredTile>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
) {
    hovered.0 = cursor_to_tile(&windows, &camera, &cfg);
}

#[allow(clippy::type_complexity)]
fn cleanup_map(mut commands: Commands, entities: Query<Entity, Or<(With<TilePosition>, With<Camera2d>)>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
