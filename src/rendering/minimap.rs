use crate::economy::components::Player;
use bevy::prelude::*;

#[derive(Component)]
pub struct MinimapCamera;

pub fn setup_minimap(mut commands: Commands) {
    commands
        .spawn((Camera2d, MinimapCamera))
        .insert(Camera {
            order: 1,
            viewport: Some(bevy::camera::Viewport {
                physical_position: UVec2::ZERO,
                physical_size: UVec2::new(200, 200),
                ..default()
            }),
            ..default()
        })
        .insert(Projection::Orthographic(OrthographicProjection {
            scale: 10.0,
            ..OrthographicProjection::default_2d()
        }));
}

pub fn update_minimap(
    window: Query<&Window>,
    player: Query<&Transform, (With<Player>, Without<MinimapCamera>)>,
    mut minimap: Query<(&mut Transform, &mut Camera), With<MinimapCamera>>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Ok((mut tf, mut camera)) = minimap.single_mut() else {
        return;
    };

    let margin = 10u32;
    let size = UVec2::new(200, 200);
    let pos = UVec2::new(
        window
            .resolution
            .physical_width()
            .saturating_sub(size.x + margin),
        window
            .resolution
            .physical_height()
            .saturating_sub(size.y + margin),
    );

    camera.viewport = Some(bevy::camera::Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });

    if let Ok(player_tf) = player.single() {
        tf.translation.x = player_tf.translation.x;
        tf.translation.y = player_tf.translation.y;
    }
}
