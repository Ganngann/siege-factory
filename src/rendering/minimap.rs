// Minimap rendering.

use crate::economy::components::Player;
use crate::ui::components::minimap::MinimapBorderConfig;
use bevy::prelude::*;

#[derive(Component)]
pub struct MinimapCamera;

pub fn update_minimap(
    window: Query<&Window>,
    player: Query<&Transform, (With<Player>, Without<MinimapCamera>)>,
    mut minimap: Query<(&mut Transform, &mut Camera, &MinimapBorderConfig), With<MinimapCamera>>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Ok((mut tf, mut camera, cfg)) = minimap.single_mut() else {
        return;
    };

    let size_u = UVec2::new(cfg.size as u32, cfg.size as u32);
    let pos = UVec2::new(
        window
            .resolution
            .physical_width()
            .saturating_sub(size_u.x + cfg.margin as u32),
        window
            .resolution
            .physical_height()
            .saturating_sub(size_u.y + cfg.margin as u32),
    );

    camera.viewport = Some(bevy::camera::Viewport {
        physical_position: pos,
        physical_size: size_u,
        ..default()
    });

    if let Ok(player_tf) = player.single() {
        tf.translation.x = player_tf.translation.x;
        tf.translation.y = player_tf.translation.y;
    }
}
