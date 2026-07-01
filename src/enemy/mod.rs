pub mod registry;
pub mod wave_config;

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use crate::combat::Projectile;
use crate::core::game_state::GameState;
use crate::economy::building::BuildingRegistry;
use crate::economy::systems::{HQ, Building, Turret};
use crate::economy::resource::Inventory;
use crate::events::DespawnEnemy;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use crate::rendering::{material_from_color, ShapeCache};
use registry::EnemyRegistry;
use wave_config::WaveConfig;
use std::collections::VecDeque;

#[derive(Component)]
struct WaveCounterText;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveState::default());
        app.insert_resource(EnemyRegistry::load());
        app.insert_resource(WaveConfig::load());
        app.add_systems(OnEnter(GameState::Playing), reset_wave);
        app.add_systems(OnExit(GameState::Playing), cleanup_game_entities);
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui);
        app.add_systems(OnExit(GameState::GameOver), despawn_game_over_ui);
        app.add_systems(Update, (
            wave_timer,
            spawn_enemies,
            move_enemies,
            enemies_damage_hq,
            turret_shoot,
            wave_counter_ui,
        ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
pub struct WaveState {
    pub timer: f32,
    pub wave: u32,
    pub spawn_timer: f32,
    pub enemies_this_wave: u32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            timer: 3.0,
            wave: 1,
            spawn_timer: 0.0,
            enemies_this_wave: 0,
        }
    }
}

#[derive(Component)]
struct GameOverUi;

// ── Game Over / Victory screen ──

fn spawn_game_over_ui(
    mut commands: Commands,
    wave: Res<WaveState>,
    cfg: Res<WaveConfig>,
) {
    let won = wave.wave > cfg.win_waves;
    commands.spawn((Camera2dBundle::default(), GameOverUi));
    commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        }, GameOverUi))
        .with_children(|parent| {
            parent.spawn((TextBundle::from_section(
                if won { "VICTORY" } else { "GAME OVER" },
                TextStyle {
                    font_size: 48.0,
                    color: if won { Color::srgb(0.3, 1.0, 0.3) } else { Color::srgb(1.0, 0.3, 0.3) },
                    ..default()
                },
            ), GameOverUi));
            parent.spawn((TextBundle::from_section(
                if won { format!("Survived {} waves!", wave.wave - 1) }
                    else { format!("Waves survived: {}", wave.wave - 1) },
                TextStyle { font_size: 24.0, color: Color::WHITE, ..default() },
            ), GameOverUi));
            parent.spawn((TextBundle::from_section(
                "",
                TextStyle::default(),
            ), GameOverUi));
            parent.spawn((TextBundle::from_section(
                "Press R to restart",
                TextStyle { font_size: 20.0, color: Color::srgb(0.8, 0.8, 1.0), ..default() },
            ), GameOverUi));
        });
}

fn despawn_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_game_entities(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<TilePosition>)>,
    soldiers_and_workers: Query<Entity, Or<(With<crate::unit::Soldier>, With<crate::unit::Worker>)>>,
) {
    for entity in enemies.iter().chain(soldiers_and_workers.iter()) {
        commands.entity(entity).despawn();
    }
}

fn reset_wave(
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    hq: Query<Entity, With<HQ>>,
    cfg: Res<MapConfig>,
) {
    *wave = WaveState::default();
    if let Ok(entity) = hq.get_single() {
        commands.entity(entity).insert(Health { current: cfg.hq_hp, max: cfg.hq_hp });
    }
}

// ── Wave progression ──

fn wave_timer(
    time: Res<Time>,
    mut wave: ResMut<WaveState>,
    existing: Query<Entity, With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
    cfg: Res<WaveConfig>,
) {
    wave.timer -= time.delta_seconds();
    if wave.timer <= 0.0 && existing.iter().len() == 0 {
        wave.wave += 1;
        wave.timer = cfg.wave_interval_sec;
        if wave.wave > cfg.win_waves {
            next_state.set(GameState::GameOver);
        }
    }
}

// ── Enemy spawning ──

fn spawn_enemies(
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
    let grid_w = map_cfg.width;
    let grid_h = map_cfg.height;

    let max_enemies = (wave.wave * cfg.max_enemies_base).min(cfg.max_enemies_cap);
    if existing.iter().len() >= max_enemies as usize {
        return;
    }

    wave.spawn_timer -= time.delta_seconds();
    if wave.spawn_timer > 0.0 {
        return;
    }
    wave.spawn_timer = (cfg.spawn_interval_sec / wave.wave as f32).max(cfg.spawn_timer_min);

    let hq_pos = match hq.get_single() {
        Ok(p) => p,
        Err(_) => return,
    };

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let (sx, sy) = loop {
        let edge = rng.gen_range(0..4);
        let (x, y) = match edge {
            0 => (rng.gen_range(0..grid_w), 0),
            1 => (rng.gen_range(0..grid_w), grid_h - 1),
            2 => (0, rng.gen_range(0..grid_h)),
            _ => (grid_w - 1, rng.gen_range(0..grid_h)),
        };
        if x != hq_pos.x || y != hq_pos.y {
            break (x, y);
        }
    };

    let def = enemies_registry.get("runner").unwrap_or_else(|| {
        panic!("enemy 'runner' not found in registry")
    });
    let enemy_hp = def.hp + (wave.wave - 1) * cfg.hp_per_wave;

    commands.spawn((
        Enemy,
        Health { current: enemy_hp, max: enemy_hp },
        ColorMesh2dBundle {
            mesh: Mesh2dHandle(shapes.circle.clone()),
            material: material_from_color(&mut materials, def.color),
            transform: Transform::from_xyz(
                sx as f32 * tile_size + tile_size / 2.0,
                sy as f32 * tile_size + tile_size / 2.0,
                3.0,
            ),
            ..default()
        },
        TilePosition { x: sx, y: sy },
    ));
}

// ── Pathfinding ──

fn bfs(
    start: (u32, u32),
    goal: (u32, u32),
    blocked: &[bool],
    grid_w: u32,
    grid_h: u32,
) -> Option<Vec<(u32, u32)>> {
    let size = (grid_w * grid_h) as usize;
    let mut queue = VecDeque::new();
    let mut visited = vec![false; size];
    let mut parent = vec![None; size];

    let start_idx = start.1 as usize * grid_w as usize + start.0 as usize;
    if blocked[start_idx] {
        return None;
    }

    queue.push_back(start);
    visited[start_idx] = true;

    while let Some((cx, cy)) = queue.pop_front() {
        if cx == goal.0 && cy == goal.1 {
            let mut path = Vec::new();
            let mut cur = (goal.0, goal.1);
            while cur != start {
                path.push(cur);
                let idx = cur.1 as usize * grid_w as usize + cur.0 as usize;
                cur = parent[idx].unwrap();
            }
            path.reverse();
            return Some(path);
        }

        for (dx, dy) in [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)] {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < grid_w as i32 && ny >= 0 && ny < grid_h as i32 {
                let nx = nx as u32;
                let ny = ny as u32;
                let idx = ny as usize * grid_w as usize + nx as usize;
                if !visited[idx] && !blocked[idx] {
                    visited[idx] = true;
                    parent[idx] = Some((cx, cy));
                    queue.push_back((nx, ny));
                }
            }
        }
    }
    None
}

// ── Movement ──

fn move_enemies(
    mut set: ParamSet<(
        Query<(Entity, &mut Transform, &mut TilePosition), With<Enemy>>,
        Query<&TilePosition, With<HQ>>,
        Query<&TilePosition, (With<Building>, Without<HQ>)>,
    )>,
    time: Res<Time>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<MapConfig>,
) {
    let grid_w = cfg.width;
    let grid_h = cfg.height;
    let tile_size = cfg.tile_size;

    let enemy_speed = enemies_registry.get("runner")
        .map(|d| d.speed)
        .unwrap_or(60.0);

    let hq_pos = match set.p1().get_single() {
        Ok(p) => *p,
        Err(_) => return,
    };
    let goal = (hq_pos.x, hq_pos.y);
    let mut blocked = vec![false; (grid_w * grid_h) as usize];
    for &pos in set.p2().iter() {
        let idx = pos.y as usize * grid_w as usize + pos.x as usize;
        blocked[idx] = true;
    }

    for (_entity, mut transform, mut pos) in set.p0().iter_mut() {
        let start = (pos.x, pos.y);
        if start == goal {
            continue;
        }

        let path = bfs(start, goal, &blocked, grid_w, grid_h);
        let target = match path {
            Some(ref p) if !p.is_empty() => (p[0].0, p[0].1),
            _ => continue,
        };

        let target_wx = target.0 as f32 * tile_size + tile_size / 2.0;
        let target_wy = target.1 as f32 * tile_size + tile_size / 2.0;
        let dx = target_wx - transform.translation.x;
        let dy = target_wy - transform.translation.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 2.0 {
            pos.x = target.0;
            pos.y = target.1;
            transform.translation.x = target_wx;
            transform.translation.y = target_wy;
        } else {
            let step = enemy_speed * time.delta_seconds();
            let ratio = (step / dist).min(1.0);
            transform.translation.x += dx * ratio;
            transform.translation.y += dy * ratio;
        }
    }
}

// ── Combat ──

fn enemies_damage_hq(
    enemies: Query<(Entity, &TilePosition), With<Enemy>>,
    mut hq: Query<(&mut Health, &mut Inventory), With<HQ>>,
    mut next_state: ResMut<NextState<GameState>>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<MapConfig>,
    mut enemy_events: EventWriter<DespawnEnemy>,
) {
    let enemy_damage = enemies_registry.get("runner")
        .map(|d| d.damage)
        .unwrap_or(10);

    let (mut hq_health, _inv) = match hq.get_single_mut() {
        Ok(h) => h,
        Err(_) => return,
    };

    let hq_tx = cfg.width / 2;
    let hq_ty = cfg.height / 2;

    for (entity, pos) in enemies.iter() {
        if pos.x == hq_tx && pos.y == hq_ty {
            enemy_events.send(DespawnEnemy(entity));
            hq_health.current = hq_health.current.saturating_sub(enemy_damage);
        }
    }

    if hq_health.current == 0 {
        next_state.set(GameState::GameOver);
    }
}

fn turret_shoot(
    mut commands: Commands,
    mut turrets: Query<(&Transform, &mut Turret)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
    buildings: Res<BuildingRegistry>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let turret_def = match buildings.get("turret") {
        Some(d) => d,
        None => return,
    };
    let combat = match &turret_def.combat {
        Some(c) => c,
        None => return,
    };
    let range_sq = combat.range.ceil() as u32 as f32;
    let damage = combat.damage;
    let fire_interval = combat.fire_rate_sec;

    for (turret_pos, mut turret) in turrets.iter_mut() {
        turret.fire_timer += time.delta_seconds();
        if turret.fire_timer < fire_interval {
            continue;
        }

        let mut target = None;
        let mut closest_dist = range_sq;

        for (entity, enemy_pos) in enemies.iter() {
            let dist = enemy_pos.translation.distance_squared(turret_pos.translation);
            if dist < closest_dist {
                closest_dist = dist;
                target = Some(entity);
            }
        }

        if let Some(entity) = target {
            turret.fire_timer -= fire_interval;
            commands.spawn((
                Projectile {
                    target: entity,
                    speed: 300.0,
                    damage,
                },
                ColorMesh2dBundle {
                    mesh: Mesh2dHandle(shapes.circle.clone()),
                    material: material_from_color(&mut materials, Color::srgb(1.0, 0.8, 0.2)),
                    transform: Transform::from_translation(turret_pos.translation).with_scale(Vec3::splat(0.3)),
                    ..default()
                },
            ));
        }
    }
}

fn wave_counter_ui(
    wave: Res<WaveState>,
    enemies: Query<Entity, With<Enemy>>,
    cfg: Res<WaveConfig>,
    mut text_query: Query<(Entity, &mut Text), With<WaveCounterText>>,
    mut commands: Commands,
) {
    let count = enemies.iter().len();
    let msg = format!("Wave {}/{}  |  Enemies: {}", wave.wave, cfg.win_waves, count);

    if let Ok((_, mut text)) = text_query.get_single_mut() {
        text.sections[0].value = msg;
    } else {
        commands.spawn((
            WaveCounterText,
            TextBundle {
                text: Text::from_sections([TextSection::new(
                    msg,
                    TextStyle { font_size: 16.0, color: Color::srgb(1.0, 0.6, 0.2), ..default() },
                )]),
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
        ));
    }
}
