use bevy::prelude::*;
use serde::Deserialize;

use crate::core::tutorial::{TutorialConditions, TutorialState};
use crate::core::toast::ToastQueue;
use crate::economy::game_components::CurrentTier;

// ── Data types ──

#[derive(Debug, Clone)]
pub struct ObjectiveDef {
    pub id: String,
    pub text: String,
    pub trigger_type: String,
    pub trigger_id: String,
}

#[derive(Debug, Clone, Resource)]
pub struct ObjectiveRegistry {
    pub objectives: Vec<ObjectiveDef>,
}

impl ObjectiveRegistry {
    pub fn load(mods: &crate::core::modding::ModRegistry) -> Self {
        let mut objectives = Vec::new();
        if let Some(content) = mods.load_data("objectives.toml") {
            if let Ok(parsed) = toml::from_str::<ObjectivesToml>(&content) {
                for entry in parsed.objectives {
                    objectives.push(ObjectiveDef {
                        id: entry.id,
                        text: entry.text,
                        trigger_type: entry.trigger_type,
                        trigger_id: entry.trigger_id,
                    });
                }
            }
        }
        Self { objectives }
    }
}

#[derive(Deserialize)]
struct ObjectivesToml {
    #[serde(default)]
    objectives: Vec<ObjectiveEntry>,
}

#[derive(Deserialize)]
struct ObjectiveEntry {
    id: String,
    text: String,
    trigger_type: String,
    #[serde(default)]
    trigger_id: String,
}

// ── Runtime state ──

#[derive(Resource, Default)]
pub struct ObjectiveState {
    pub current_index: usize,
    pub active_text: String,
}

// ── Marker pour le HUD ──

#[derive(Component)]
pub struct ObjectiveHudMarker;

// ── Systems ──

pub fn advance_objectives(
    mut state: ResMut<ObjectiveState>,
    registry: Res<ObjectiveRegistry>,
    conditions: Res<TutorialConditions>,
    tutorial: Res<TutorialState>,
    capsule_q: Query<&CurrentTier>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    if registry.objectives.is_empty() || state.current_index >= registry.objectives.len() {
        return;
    }

    let obj = &registry.objectives[state.current_index];
    let done = match obj.trigger_type.as_str() {
        "game_start" => {
            // Always complete on first tick → advance to next objective
            state.current_index += 1;
            true
        }
        "tutorial_step" => {
            // Find the objective's step position in the tutorial
            let step_pos = tutorial
                .steps
                .iter()
                .position(|s| s.id == obj.trigger_id);
            match step_pos {
                Some(pos) => tutorial.current_index > pos || tutorial.completed,
                None => false,
            }
        }
        "item_crafted" => {
            conditions
                .items_crafted
                .get(&obj.trigger_id)
                .copied()
                .unwrap_or(0)
                >= 1
        }
        "tier_unlocked" => {
            // Check if the capsule's current tier has reached the required index
            if let Ok(tier) = capsule_q.single() {
                // trigger_id is the log_id of the tier (e.g. "genesis_phase_0")
                // Find the tier index from the log_id — advance if we're past that tier
                // For simplicity: advance if current_index == some index based on order
                // We use ordinal position in the objectives list as the tier index
                // since objectives are ordered in the TOML
                let tier_idx = registry
                    .objectives
                    .iter()
                    .position(|o| o.id == obj.id)
                    .unwrap_or(0);
                tier.0 > tier_idx
            } else {
                false
            }
        }
        _ => false,
    };

    if done {
        state.current_index += 1;
        if state.current_index < registry.objectives.len() {
            let next = &registry.objectives[state.current_index];
            state.active_text = next.text.clone();
            toast_queue.push(format!("🎯 {}", next.text));
        } else {
            state.active_text = String::new();
        }
    }
}

pub fn spawn_objective_hud(mut commands: Commands, state: Res<ObjectiveState>) {
    if state.active_text.is_empty() {
        return;
    }

    commands
        .spawn((
            ObjectiveHudMarker,
            Text::new(format!("OBJECTIF\n{}", state.active_text)),
            TextFont::from_font_size(14.0),
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Percent(-25.0)),
                width: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ZIndex(50),
        ));
}

pub fn update_objective_hud(
    state: Res<ObjectiveState>,
    mut query: Query<&mut Text, With<ObjectiveHudMarker>>,
) {
    for mut text in query.iter_mut() {
        if state.active_text.is_empty() {
            text.0 = String::new();
        } else {
            text.0 = format!("OBJECTIF\n{}", state.active_text);
        }
    }
}

pub fn despawn_objective_hud(mut commands: Commands, query: Query<Entity, With<ObjectiveHudMarker>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
