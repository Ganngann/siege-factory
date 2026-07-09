use crate::core::utils::silent_despawn;
use crate::economy::components::PeacefulMode;
use crate::enemy::components::{Enemy, GameOverUi, WaveCounterText, WaveState};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

fn spawn_hud_node(
    commands: &mut Commands,
    text: &str,
    font_size: f32,
    color: Color,
    x: f32,
    y: f32,
) -> Entity {
    commands
        .spawn((
            Text::new(text),
            TextFont::from_font_size(font_size),
            TextColor(color),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                ..default()
            },
        ))
        .id()
}

pub fn wave_counter_ui(
    wave: Res<WaveState>,
    enemies: Query<Entity, With<Enemy>>,
    peaceful: Res<PeacefulMode>,
    mut text_query: Query<(Entity, &mut Text), With<WaveCounterText>>,
    mut commands: Commands,
) {
    let msg = if peaceful.0 {
        "Peaceful Mode  |  No enemies".to_string()
    } else {
        let count = enemies.iter().len();
        format!("Wave {}  |  Enemies: {}", wave.wave, count)
    };

    if let Ok((_, mut text)) = text_query.single_mut() {
        text.0 = msg;
    } else {
        let entity = spawn_hud_node(
            &mut commands,
            &msg,
            16.0,
            Color::srgb(1.0, 0.6, 0.2),
            0.0,
            10.0,
        );
        commands.entity(entity).insert(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        });
        commands.entity(entity).insert(WaveCounterText);
    }
}

pub fn spawn_game_over_ui(mut commands: Commands, wave: Res<WaveState>) {
    commands.spawn((Camera2d, GameOverUi));
    commands
        .spawn((
            GameOverUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                GameOverUi,
                Text::new("GAME OVER"),
                TextFont::from_font_size(48.0),
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));
            parent.spawn((
                GameOverUi,
                Text::new(format!("Waves survived: {}", wave.wave - 1)),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                GameOverUi,
                Text::new(""),
                TextFont::default(),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                GameOverUi,
                Text::new("Press R to restart  |  ESC for main menu"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
            ));
        });
}

pub fn despawn_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUi>>) {
    for entity in &query {
        silent_despawn(&mut commands, entity);
    }
}

#[derive(Component)]
pub struct FpsOverlay;

#[derive(Resource)]
pub struct FpsUpdateTimer(pub Timer);

impl Default for FpsUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

pub fn fps_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mut text_query: Query<(Entity, &mut Text), With<FpsOverlay>>,
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<FpsUpdateTimer>,
) {
    timer.0.tick(time.delta());

    if text_query.single_mut().is_err() {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.smoothed())
            .map_or("--".to_string(), |v| format!("{:.0}", v));
        let entity = spawn_hud_node(
            &mut commands,
            &format!("FPS: {}", fps),
            14.0,
            Color::srgb(0.0, 1.0, 0.0),
            10.0,
            10.0,
        );
        commands.entity(entity).insert(FpsOverlay);
        return;
    }

    if !timer.0.just_finished() {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .map_or("--".to_string(), |v| format!("{:.0}", v));

    if let Ok((_, mut text)) = text_query.single_mut() {
        text.0 = format!("FPS: {}", fps);
    }
}
