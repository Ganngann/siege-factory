use bevy::prelude::*;

use crate::core::toast::ToastQueue;
use crate::economy::components::Player;
use crate::economy::game_components::{DataPad, DataPadRead};
use crate::economy::tiered_structure::ProgressionLogRegistry;
use crate::map::components::TilePosition;

pub fn interact_data_pad(
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<&TilePosition, With<Player>>,
    mut commands: Commands,
    data_pads: Query<(Entity, &DataPad, &TilePosition, Option<&DataPadRead>)>,
    mut progression_logs: ResMut<ProgressionLogRegistry>,
    mut toasts: ResMut<ToastQueue>,
) {
    // ⚠️ IA ATTENTION: KeyE en dur. Devrait utiliser le système KeyBindings.
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player_tile) = player_q.single() else {
        return;
    };

    let mut to_read: Vec<(Entity, String)> = Vec::new();
    for (pad_entity, pad, pad_tile, already_read) in &data_pads {
        let dx = (player_tile.x - pad_tile.x).abs();
        let dy = (player_tile.y - pad_tile.y).abs();
        if dx > 1 || dy > 1 {
            continue;
        }
        if already_read.is_some() {
            toasts.0.push("This data pad has already been read.".to_string());
            continue;
        }
        to_read.push((pad_entity, pad.log_id.clone()));
    }

    for (pad_entity, log_id) in to_read {
        if let Some(entry) = progression_logs.unlock(&log_id) {
            toasts
                .0
                .push(format!("Log: {} — {}", entry.title, entry.text));
        }
        commands.entity(pad_entity).insert(DataPadRead);
    }
}
