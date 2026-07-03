use bevy::prelude::*;

use crate::core::game_state::GameState;
use crate::core::toast::ToastMessage;
use crate::economy::components::HQ;
use crate::enemy::components::{Enemy, WaveState, WaveCounterText, GameOverUi, Health, LastWave};
use crate::enemy::registry::EnemyRegistry;
use crate::enemy::wave_config::WaveConfig;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{material_from_color, ShapeCache};

pub fn wave_timer(
    time: Res<Time>,
    mut wave: ResMut<WaveState>,
    existing: Query<Entity, With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
    cfg: Res<WaveConfig>,
) {
    wave.timer -= time.delta_secs();
    if wave.timer <= 0.0 && existing.iter().len() == 0 {
        wave.wave += 1;
        wave.timer = cfg.wave_interval_sec;
        if wave.wave > cfg.win_waves {
            next_state.set(GameState::GameOver);
        }
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    time: Res<Time>,
    hq: Query<&TilePosition, With<HQ>>,
    existing: Query<Entity, With<Enemy>>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<WaveConfig>,
    map_cfg: Res<MapConfig>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tile_size = map_cfg.tile_size;

    let max_enemies = (wave.wave * cfg.max_enemies_base).min(cfg.max_enemies_cap);
    if existing.iter().len() >= max_enemies as usize {
        return;
    }

    wave.spawn_timer -= time.delta_secs();
    if wave.spawn_timer > 0.0 {
        return;
    }
    wave.spawn_timer = (cfg.spawn_interval_sec / wave.wave as f32).max(cfg.spawn_timer_min);

    let hq_pos = match hq.single() {
        Ok(p) => p,
        Err(_) => return,
    };

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let spawn_dist = 25.0;
    let sx = (hq_pos.x as f32 + angle.cos() * spawn_dist).round() as i32;
    let sy = (hq_pos.y as f32 + angle.sin() * spawn_dist).round() as i32;

    let def = enemies_registry.get("runner").unwrap_or_else(|| {
        panic!("enemy 'runner' not found in registry")
    });
    let enemy_hp = def.hp + (wave.wave - 1) * cfg.hp_per_wave;

    commands.spawn((
        Enemy,
        Health { current: enemy_hp, max: enemy_hp },
        Mesh2d(shapes.circle.clone()),
        MeshMaterial2d(material_from_color(&mut materials, def.color)),
        Transform::from_xyz(
            sx as f32 * tile_size + tile_size / 2.0,
            sy as f32 * tile_size + tile_size / 2.0,
            3.0,
        ),
        TilePosition { x: sx, y: sy },
    ));
}

pub fn wave_counter_ui(
    wave: Res<WaveState>,
    enemies: Query<Entity, With<Enemy>>,
    cfg: Res<WaveConfig>,
    mut text_query: Query<(Entity, &mut Text), With<WaveCounterText>>,
    mut commands: Commands,
) {
    let count = enemies.iter().len();
    let msg = format!("Wave {}/{}  |  Enemies: {}", wave.wave, cfg.win_waves, count);

    if let Ok((_, mut text)) = text_query.single_mut() {
        text.0 = msg;
    } else {
        commands.spawn((
            WaveCounterText,
            Text::new(msg),
            TextFont::from_font_size(16.0),
            TextColor(Color::srgb(1.0, 0.6, 0.2)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
        ));
    }
}

pub fn spawn_game_over_ui(
    mut commands: Commands,
    wave: Res<WaveState>,
    cfg: Res<WaveConfig>,
) {
    let won = wave.wave > cfg.win_waves;
    commands.spawn((Camera2d, GameOverUi));
    commands
        .spawn((GameOverUi, Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }))
        .with_children(|parent| {
            parent.spawn((
                GameOverUi,
                Text::new(if won { "VICTORY" } else { "GAME OVER" }),
                TextFont::from_font_size(48.0),
                TextColor(if won { Color::srgb(0.3, 1.0, 0.3) } else { Color::srgb(1.0, 0.3, 0.3) }),
            ));
            parent.spawn((
                GameOverUi,
                Text::new(if won { format!("Survived {} waves!", wave.wave - 1) }
                    else { format!("Waves survived: {}", wave.wave - 1) }),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));
            parent.spawn((GameOverUi, Text::new(""), TextFont::default(), TextColor(Color::WHITE)));
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
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_game_entities(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<TilePosition>)>,
    units: Query<Entity, With<crate::economy::components::Unit>>,
) {
    for entity in enemies.iter().chain(units.iter()) {
        commands.entity(entity).despawn();
    }
}

pub fn reset_wave(
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    hq: Query<Entity, With<HQ>>,
    cfg: Res<MapConfig>,
) {
    *wave = WaveState::default();
    if let Ok(entity) = hq.single() {
        commands.entity(entity).insert(Health { current: cfg.hq_hp, max: cfg.hq_hp });
    }
}

pub fn wave_announcement(
    wave: Res<WaveState>,
    mut last_wave: ResMut<LastWave>,
    mut commands: Commands,
) {
    if wave.wave != last_wave.0 && wave.wave > 1 {
        commands.spawn((
            ToastMessage { timer: 2.0 },
            Text::new(format!("Wave {}", wave.wave)),
            TextFont::from_font_size(32.0),
            TextColor(Color::srgb(1.0, 0.6, 0.2)),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                left: Val::Percent(50.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));
        last_wave.0 = wave.wave;
    }
}
