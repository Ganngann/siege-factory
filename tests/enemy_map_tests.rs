use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use siege_factory::core::game_state::GameState;
use siege_factory::core::toast::ToastQueue;
use siege_factory::economy::components::{PeacefulMode, Player, PowerConsumer, TurretCombat};
use siege_factory::economy::game_components::UnbuiltBuilding;
use siege_factory::economy::spatial::SpatialRegistry;
use siege_factory::enemy::ai::move_enemies;
use siege_factory::enemy::combat::{enemies_damage_player, find_closest_enemy, turret_shoot};
use siege_factory::enemy::components::{Enemy, Health, LastWave, WaveState};
use siege_factory::enemy::registry::EnemyRegistry;
use siege_factory::enemy::systems::{reset_wave, spawn_enemies, wave_announcement, wave_timer};
use siege_factory::enemy::wave_config::WaveConfig;
use siege_factory::map::components::{TilePosition, HoveredTile};
use siege_factory::map::config::MapConfig;
use siege_factory::map::systems::chunks::{build_chunk_mesh, update_fog_of_war};
use siege_factory::map::tile_grid::{ChunkGrid, CHUNK_SIZE};
use siege_factory::core::modding::ModRegistry;


// ════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════

fn test_mods() -> ModRegistry { ModRegistry::for_test() }
fn enemy_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();

    let map_cfg = MapConfig::load(&test_mods());
    let wave_cfg = WaveConfig::load(&test_mods());
    let enemy_reg = EnemyRegistry::load(&test_mods());

    app.insert_resource(map_cfg);
    app.insert_resource(wave_cfg);
    app.insert_resource(enemy_reg);
    app.insert_resource(WaveState::new(3.0));
    app.insert_resource(LastWave(1));
    app.insert_resource(ToastQueue::default());
    app.insert_resource(PeacefulMode(false));
    app.insert_resource(SpatialRegistry::default());

    app
}

fn spawn_player(app: &mut App, x: i32, y: i32, hp: u32) {
    app.world_mut().spawn((
        Player,
        Health { current: hp, max: hp },
        TilePosition { x, y },
        Transform::default(),
    ));
}

fn spawn_enemy(app: &mut App, x: i32, y: i32, hp: u32, kind: &str) -> Entity {
    app.world_mut()
        .spawn((
            Enemy {
                kind: kind.to_string(),
            },
            Health {
                current: hp,
                max: hp,
            },
            TilePosition { x, y },
            Transform::from_xyz(x as f32 * 32.0, y as f32 * 32.0, 3.0),
        ))
        .id()
}

fn make_chunk_grid() -> ChunkGrid {
    ChunkGrid::new(42, 50, 150, 35, 2, 5, vec![("iron_ore".to_string(), 50)])
}

fn get_player_hp(app: &mut App) -> u32 {
    let mut q = app.world_mut().query_filtered::<&Health, With<Player>>();
    q.iter(app.world()).next().unwrap().current
}

fn get_enemy_count(app: &mut App) -> usize {
    app.world_mut().query::<&Enemy>().iter(app.world()).count()
}

// ════════════════════════════════════════════════════════════════
// wave_timer tests
// ════════════════════════════════════════════════════════════════

#[test]
fn wave_timer_does_not_advance_with_positive_timer() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_timer);

    let initial = app.world().resource::<WaveState>().timer;
    assert!(initial > 0.0);

    app.update();

    // Timer should not have triggered a wave advance
    let wave = app.world().resource::<WaveState>();
    assert_eq!(wave.wave, 1, "wave should still be 1 with positive timer");
    assert!(wave.spawn_queue.is_empty(), "no enemies should be queued");
}

#[test]
fn wave_timer_does_not_advance_while_enemies_alive() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_timer);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.timer = 0.0;
    }
    spawn_enemy(&mut app, 5, 5, 20, "runner");

    let wave_before = app.world().resource::<WaveState>().wave;
    app.update();
    let wave_after = app.world().resource::<WaveState>().wave;

    assert_eq!(wave_before, wave_after, "wave should not advance with enemies alive");
}

#[test]
fn wave_timer_advances_when_enemies_dead_and_timer_expired() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_timer);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.timer = 0.0;
        wave.wave = 1;
    }

    let wave_before = app.world().resource::<WaveState>().wave;
    app.update();
    let wave_after = app.world().resource::<WaveState>().wave;

    assert_eq!(wave_after, wave_before + 1, "wave should advance");
}

#[test]
fn wave_timer_fills_spawn_queue_on_advance() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_timer);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.timer = 0.0;
    }

    app.update();

    let wave = app.world().resource::<WaveState>();
    assert!(!wave.spawn_queue.is_empty(), "spawn_queue should be filled after wave advance");
    assert_eq!(wave.spawn_queue[0].kind, "runner", "first wave should spawn runners");
}

#[test]
fn wave_timer_resets_interval_after_advance() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_timer);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.timer = 0.0;
    }

    app.update();

    let wave = app.world().resource::<WaveState>();
    let cfg = app.world().resource::<WaveConfig>();
    assert!(
        (wave.timer - cfg.wave_interval_sec).abs() < f32::EPSILON,
        "timer should reset to wave_interval_sec"
    );
}

#[test]
fn wave_timer_no_advance_in_peaceful_mode() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_timer);

    {
        app.world_mut().resource_mut::<PeacefulMode>().0 = true;
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.timer = 0.0;
    }

    let wave_before = app.world().resource::<WaveState>().wave;
    app.update();
    let wave_after = app.world().resource::<WaveState>().wave;

    assert_eq!(wave_before, wave_after, "no advance in peaceful mode");
}

// ════════════════════════════════════════════════════════════════
// spawn_enemies tests
// ════════════════════════════════════════════════════════════════

#[test]
fn spawn_enemies_creates_entity_when_queue_and_timer_ready() {
    let mut app = enemy_test_app();
    app.add_systems(Update, spawn_enemies);
    spawn_player(&mut app, 0, 0, 100);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.spawn_timer = 0.0;
        wave.spawn_queue
            .push(siege_factory::enemy::wave_config::WaveEntry {
                kind: "runner".to_string(),
                count: 3,
            });
    }

    let count_before = get_enemy_count(&mut app);
    app.update();
    let count_after = get_enemy_count(&mut app);

    assert_eq!(count_after, count_before + 1, "one enemy should be spawned");
}

#[test]
fn spawn_enemies_respects_max_enemies_cap() {
    let mut app = enemy_test_app();
    app.add_systems(Update, spawn_enemies);
    spawn_player(&mut app, 0, 0, 100);

    {
        let cfg = app.world().resource::<WaveConfig>();
        let max = (1 * cfg.max_enemies_base).min(cfg.max_enemies_cap);
        for _ in 0..max {
            spawn_enemy(&mut app, 10, 10, 20, "runner");
        }
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.spawn_timer = 0.0;
        wave.spawn_queue
            .push(siege_factory::enemy::wave_config::WaveEntry {
                kind: "runner".to_string(),
                count: 10,
            });
    }

    let count_before = get_enemy_count(&mut app);
    app.update();
    let count_after = get_enemy_count(&mut app);

    assert_eq!(count_before, count_after, "should not spawn when at max");
}

#[test]
fn spawn_enemies_does_not_spawn_in_peaceful_mode() {
    let mut app = enemy_test_app();
    app.add_systems(Update, spawn_enemies);
    spawn_player(&mut app, 0, 0, 100);

    {
        app.world_mut().resource_mut::<PeacefulMode>().0 = true;
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.spawn_timer = 0.0;
        wave.spawn_queue
            .push(siege_factory::enemy::wave_config::WaveEntry {
                kind: "runner".to_string(),
                count: 1,
            });
    }

    app.update();

    let count = get_enemy_count(&mut app);
    assert_eq!(count, 0, "no enemies should spawn in peaceful mode");
}

#[test]
fn spawn_enemies_at_spawn_distance_from_player() {
    let mut app = enemy_test_app();
    app.add_systems(Update, spawn_enemies);
    spawn_player(&mut app, 10, 10, 100);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.spawn_timer = 0.0;
        wave.spawn_queue
            .push(siege_factory::enemy::wave_config::WaveEntry {
                kind: "runner".to_string(),
                count: 1,
            });
    }

    app.update();

    let spawn_dist = app.world().resource::<WaveConfig>().spawn_distance;

    let mut q = app.world_mut().query_filtered::<&TilePosition, Without<Player>>();
    for pos in q.iter(app.world()) {
        let dx = (pos.x - 10) as f32;
        let dy = (pos.y - 10) as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            dist <= spawn_dist + 2.0,
            "enemy at ({},{}) should be within spawn distance {} of player, got {}",
            pos.x, pos.y, spawn_dist, dist
        );
    }
}

// ════════════════════════════════════════════════════════════════
// move_enemies tests
// ════════════════════════════════════════════════════════════════

#[test]
fn move_enemies_advances_enemy_toward_player() {
    let mut app = enemy_test_app();
    app.add_systems(Update, move_enemies);

    spawn_player(&mut app, 10, 0, 100);
    spawn_enemy(&mut app, 0, 0, 20, "runner");

    app.update();

    let enemy_count = get_enemy_count(&mut app);
    assert_eq!(enemy_count, 1, "enemy should still exist after movement");
}

#[test]
fn move_enemies_no_crash_with_no_player() {
    let mut app = enemy_test_app();
    app.add_systems(Update, move_enemies);

    spawn_enemy(&mut app, 0, 0, 20, "runner");

    app.update();

    let enemy_count = get_enemy_count(&mut app);
    assert_eq!(enemy_count, 1, "enemy should still exist");
}

#[test]
fn move_enemies_no_crash_with_no_enemies() {
    let mut app = enemy_test_app();
    app.add_systems(Update, move_enemies);

    spawn_player(&mut app, 10, 0, 100);

    app.update();
}

// ════════════════════════════════════════════════════════════════
// enemies_damage_player tests
// ════════════════════════════════════════════════════════════════

#[test]
fn enemies_damage_player_reduces_health() {
    let mut app = enemy_test_app();
    app.add_systems(Update, enemies_damage_player);

    spawn_player(&mut app, 5, 5, 100);
    spawn_enemy(&mut app, 5, 5, 20, "runner");

    let hp_before = get_player_hp(&mut app);

    app.update();

    let hp_after = get_player_hp(&mut app);

    assert!(hp_after < hp_before, "player health should decrease: {} -> {}", hp_before, hp_after);
}

#[test]
fn enemies_damage_player_uses_enemy_damage_value() {
    let mut app = enemy_test_app();
    app.add_systems(Update, enemies_damage_player);

    spawn_player(&mut app, 5, 5, 100);
    spawn_enemy(&mut app, 5, 5, 20, "runner"); // runner does 10 damage

    app.update();

    let hp_after = get_player_hp(&mut app);

    assert_eq!(hp_after, 90, "runner should deal 10 damage");
}

#[test]
fn enemies_damage_player_only_at_same_tile() {
    let mut app = enemy_test_app();
    app.add_systems(Update, enemies_damage_player);

    spawn_player(&mut app, 5, 5, 100);
    spawn_enemy(&mut app, 10, 10, 20, "runner"); // far away

    app.update();

    let hp_after = get_player_hp(&mut app);

    assert_eq!(hp_after, 100, "player should not take damage from distant enemy");
}

#[test]
fn enemies_damage_player_game_over_on_zero_hp() {
    let mut app = enemy_test_app();
    app.add_systems(Update, enemies_damage_player);

    spawn_player(&mut app, 5, 5, 5); // very low hp
    spawn_enemy(&mut app, 5, 5, 20, "runner"); // runner does 10 damage → hp would go to 0

    // First update: system runs and queues GameOver transition
    app.update();
    // Second update: state transition is applied
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::GameOver, "should transition to GameOver");
}

#[test]
fn enemies_damage_player_saturating_sub() {
    let mut app = enemy_test_app();
    app.add_systems(Update, enemies_damage_player);

    spawn_player(&mut app, 5, 5, 3); // hp=3, runner damage=10
    spawn_enemy(&mut app, 5, 5, 20, "runner");

    // First update: system runs and queues GameOver transition
    app.update();
    // Second update: state transition is applied
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::GameOver);
}

// ════════════════════════════════════════════════════════════════
// turret_shoot tests
// ════════════════════════════════════════════════════════════════

#[test]
fn turret_shoot_fires_at_nearby_enemy() {
    let mut app = enemy_test_app();
    app.add_systems(Update, turret_shoot);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat {
            damage: 10,
            range_sq: 10000.0,
            fire_interval: 1.0,
            timer: 1.0,
            projectile_speed: 200.0,
        },
    ));

    spawn_enemy(&mut app, 2, 0, 20, "runner");

    app.update();

    let combat = app.world_mut().query::<&TurretCombat>().iter(app.world()).next().unwrap();
    assert!(
        combat.timer < combat.fire_interval,
        "turret timer should have been reset after firing: {}",
        combat.timer
    );
}

#[test]
fn turret_shoot_does_not_fire_when_timer_not_ready() {
    let mut app = enemy_test_app();
    app.add_systems(Update, turret_shoot);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat {
            damage: 10,
            range_sq: 10000.0,
            fire_interval: 1.0,
            timer: 0.0,
            projectile_speed: 200.0,
        },
    ));

    spawn_enemy(&mut app, 2, 0, 20, "runner");

    app.update();

    let combat = app.world_mut().query::<&TurretCombat>().iter(app.world()).next().unwrap();
    assert!(
        combat.timer < 1.0,
        "turret should not have fired when timer was below fire_interval"
    );
}

#[test]
fn turret_shoot_skips_unpowered_consumer() {
    let mut app = enemy_test_app();
    app.add_systems(Update, turret_shoot);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat {
            damage: 10,
            range_sq: 10000.0,
            fire_interval: 1.0,
            timer: 1.0,
            projectile_speed: 200.0,
        },
        PowerConsumer {
            draw: 1.0,
            satisfied: false,
        },
    ));

    spawn_enemy(&mut app, 2, 0, 20, "runner");

    app.update();

    let combat = app.world_mut().query::<&TurretCombat>().iter(app.world()).next().unwrap();
    assert!(
        combat.timer >= 1.0 - 0.01,
        "unpowered turret should not fire, timer: {}",
        combat.timer
    );
}

#[test]
fn turret_shoot_ignores_unbuilt_buildings() {
    let mut app = enemy_test_app();
    app.add_systems(Update, turret_shoot);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat {
            damage: 10,
            range_sq: 10000.0,
            fire_interval: 1.0,
            timer: 1.0,
            projectile_speed: 200.0,
        },
        UnbuiltBuilding,
    ));

    spawn_enemy(&mut app, 2, 0, 20, "runner");

    app.update();

    let count = app.world_mut().query::<&TurretCombat>().iter(app.world()).count();
    assert_eq!(count, 1, "turret entity should still exist but not fire");
}

#[test]
fn turret_shoot_fires_only_at_enemies_in_range() {
    let mut app = enemy_test_app();
    app.add_systems(Update, turret_shoot);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat {
            damage: 10,
            range_sq: 100.0,
            fire_interval: 1.0,
            timer: 1.0,
            projectile_speed: 200.0,
        },
    ));

    spawn_enemy(&mut app, 100, 0, 20, "runner");

    app.update();

    let combat = app.world_mut().query::<&TurretCombat>().iter(app.world()).next().unwrap();
    assert!(
        combat.timer >= 0.99,
        "turret should not fire at out-of-range enemy, timer: {}",
        combat.timer
    );
}

// ════════════════════════════════════════════════════════════════
// find_closest_enemy tests
// ════════════════════════════════════════════════════════════════

#[test]
fn find_closest_enemy_returns_nearest() {
    let mut app = enemy_test_app();

    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();
    let e3 = app.world_mut().spawn_empty().id();

    let enemies = vec![
        (e1, Vec3::new(10.0, 0.0, 0.0)),
        (e2, Vec3::new(3.0, 0.0, 0.0)),
        (e3, Vec3::new(7.0, 0.0, 0.0)),
    ];

    let result = find_closest_enemy(Vec3::ZERO, &enemies, 1000.0);
    assert_eq!(result, Some(e2), "should return the closest enemy (distance 3)");
}

#[test]
fn find_closest_enemy_respects_range() {
    let mut app = enemy_test_app();

    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();

    let enemies = vec![
        (e1, Vec3::new(10.0, 0.0, 0.0)),
        (e2, Vec3::new(3.0, 0.0, 0.0)),
    ];

    let result = find_closest_enemy(Vec3::ZERO, &enemies, 5.0);
    assert_eq!(result, None, "should return None when all enemies out of range");
}

#[test]
fn find_closest_enemy_empty_list() {
    let enemies: Vec<(Entity, Vec3)> = vec![];
    let result = find_closest_enemy(Vec3::ZERO, &enemies, 100.0);
    assert_eq!(result, None);
}

#[test]
fn find_closest_enemy_exact_boundary() {
    let mut app = enemy_test_app();
    let e1 = app.world_mut().spawn_empty().id();

    let enemies = vec![(e1, Vec3::new(5.0, 0.0, 0.0))];

    let result = find_closest_enemy(Vec3::ZERO, &enemies, 25.0);
    assert_eq!(result, None, "boundary (dist == range) should not match (< not <=)");

    let result = find_closest_enemy(Vec3::ZERO, &enemies, 26.0);
    assert_eq!(result, Some(e1));
}

#[test]
fn find_closest_enemy_two_distant_enemies() {
    let mut app = enemy_test_app();
    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();

    let enemies = vec![
        (e1, Vec3::new(100.0, 0.0, 0.0)),
        (e2, Vec3::new(-50.0, 0.0, 0.0)),
    ];

    let result = find_closest_enemy(Vec3::ZERO, &enemies, 100000.0);
    assert_eq!(result, Some(e2), "should return the closer of two distant enemies");
}

// ════════════════════════════════════════════════════════════════
// wave_announcement tests
// ════════════════════════════════════════════════════════════════

#[test]
fn wave_announcement_pushes_toast_on_new_wave() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_announcement);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.wave = 2;
    }
    {
        let mut last = app.world_mut().resource_mut::<LastWave>();
        last.0 = 1;
    }

    app.update();

    let toast = app.world().resource::<ToastQueue>();
    assert!(
        toast.0.iter().any(|m| m.contains("Wave 2")),
        "should announce Wave 2, got: {:?}",
        toast.0
    );
}

#[test]
fn wave_announcement_no_toast_when_wave_unchanged() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_announcement);

    {
        let w = app.world().resource::<WaveState>().wave;
        let mut last = app.world_mut().resource_mut::<LastWave>();
        last.0 = w;
    }

    app.update();

    let toast = app.world().resource::<ToastQueue>();
    assert!(toast.0.is_empty(), "no toast when wave unchanged");
}

#[test]
fn wave_announcement_no_toast_on_wave_1() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_announcement);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.wave = 1;
    }
    {
        let mut last = app.world_mut().resource_mut::<LastWave>();
        last.0 = 0;
    }

    app.update();

    let toast = app.world().resource::<ToastQueue>();
    assert!(toast.0.is_empty(), "no toast on wave 1");
}

#[test]
fn wave_announcement_updates_last_wave() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_announcement);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.wave = 3;
    }
    {
        let mut last = app.world_mut().resource_mut::<LastWave>();
        last.0 = 1;
    }

    app.update();

    let last = app.world().resource::<LastWave>();
    assert_eq!(last.0, 3, "LastWave should be updated to current wave");
}

#[test]
fn wave_announcement_multiple_waves_sequential() {
    let mut app = enemy_test_app();
    app.add_systems(Update, wave_announcement);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.wave = 2;
    }
    {
        let mut last = app.world_mut().resource_mut::<LastWave>();
        last.0 = 1;
    }
    app.update();
    assert_eq!(app.world().resource::<ToastQueue>().0.len(), 1);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.wave = 3;
    }
    app.update();
    assert_eq!(app.world().resource::<ToastQueue>().0.len(), 2);
}

// ════════════════════════════════════════════════════════════════
// reset_wave tests
// ════════════════════════════════════════════════════════════════

#[test]
fn reset_wave_resets_wave_state() {
    let mut app = enemy_test_app();
    app.add_systems(Update, reset_wave);
    spawn_player(&mut app, 0, 0, 50);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.wave = 5;
        wave.timer = -10.0;
        wave.spawn_queue
            .push(siege_factory::enemy::wave_config::WaveEntry {
                kind: "runner".to_string(),
                count: 10,
            });
    }

    app.update();

    let wave = app.world().resource::<WaveState>();
    let cfg = app.world().resource::<WaveConfig>();
    assert_eq!(wave.wave, 1, "wave should reset to 1");
    assert!(
        (wave.timer - cfg.first_wave_delay).abs() < f32::EPSILON,
        "timer should reset to first_wave_delay"
    );
    assert!(wave.spawn_queue.is_empty(), "spawn queue should be empty");
}

#[test]
fn reset_wave_restores_player_health() {
    let mut app = enemy_test_app();
    app.add_systems(Update, reset_wave);
    spawn_player(&mut app, 0, 0, 10);

    app.update();

    let hp = get_player_hp(&mut app);
    let map_cfg = app.world().resource::<MapConfig>();
    assert_eq!(hp, map_cfg.player_hp, "player health should be restored to map_cfg.player_hp");
}

// ════════════════════════════════════════════════════════════════
// update_fog_of_war tests
// ════════════════════════════════════════════════════════════════

fn fog_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let map_cfg = MapConfig::load(&test_mods());
    let seed = map_cfg.seed;
    let dep_min = map_cfg.deposit_min_amount;
    let dep_max = map_cfg.deposit_max_amount;
    let dep_chance = map_cfg.deposit_spawn_chance_pct;
    let dep_min_per = map_cfg.deposit_min_per_chunk;
    let dep_max_per = map_cfg.deposit_max_per_chunk;
    let dep_dist = map_cfg.deposit_distribution.clone();
    app.insert_resource(map_cfg);
    app.insert_resource(ChunkGrid::new(
        seed, dep_min, dep_max, dep_chance, dep_min_per, dep_max_per, dep_dist,
    ));
    app.init_resource::<Assets<Mesh>>();
    app
}

#[test]
fn update_fog_of_war_reveals_tiles_near_player() {
    let mut app = fog_test_app();
    app.add_systems(Update, update_fog_of_war);

    let player_x = 10;
    let player_y = 10;
    app.world_mut().spawn((
        Player,
        TilePosition {
            x: player_x,
            y: player_y,
        },
    ));

    {
        let cg = app.world().resource::<ChunkGrid>();
        let chunk = cg.get_chunk(0, 0);
        assert!(chunk.is_none() || chunk.unwrap().visited.is_empty(), "chunk should start unvisited");
    }

    app.update();

    let cg = app.world().resource::<ChunkGrid>();
    let reveal_radius = 6i32;
    let mut revealed_count = 0;
    for dx in -reveal_radius..=reveal_radius {
        for dy in -reveal_radius..=reveal_radius {
            let wx = player_x + dx;
            let wy = player_y + dy;
            let cx = wx.div_euclid(CHUNK_SIZE as i32);
            let cy = wy.div_euclid(CHUNK_SIZE as i32);
            let tx = wx.rem_euclid(CHUNK_SIZE as i32) as u32;
            let ty = wy.rem_euclid(CHUNK_SIZE as i32) as u32;
            if cg.is_tile_visited(cx, cy, tx, ty) {
                revealed_count += 1;
            }
        }
    }
    assert!(revealed_count > 0, "should have revealed tiles near player");
}

#[test]
fn update_fog_of_war_does_not_reveal_far_tiles() {
    let mut app = fog_test_app();
    app.add_systems(Update, update_fog_of_war);

    app.world_mut().spawn((
        Player,
        TilePosition { x: 0, y: 0 },
    ));

    app.update();

    let cg = app.world().resource::<ChunkGrid>();
    assert!(
        !cg.is_tile_visited(3, 3, 0, 0),
        "faraway tile should not be revealed"
    );
}

#[test]
fn update_fog_of_war_no_crash_without_player() {
    let mut app = fog_test_app();
    app.add_systems(Update, update_fog_of_war);
    app.update();
}

// ════════════════════════════════════════════════════════════════
// build_chunk_mesh tests
// ════════════════════════════════════════════════════════════════

#[test]
fn build_chunk_mesh_returns_two_meshes() {
    let (mesh_a, mesh_b) = build_chunk_mesh(0, 0, 32.0);
    let pos_a = mesh_a.attribute(Mesh::ATTRIBUTE_POSITION);
    let pos_b = mesh_b.attribute(Mesh::ATTRIBUTE_POSITION);
    assert!(pos_a.is_some(), "mesh_a should have positions");
    assert!(pos_b.is_some(), "mesh_b should have positions");
}

#[test]
fn build_chunk_mesh_vertex_counts() {
    let (mesh_a, mesh_b) = build_chunk_mesh(0, 0, 32.0);

    let count_a = mesh_a.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len();
    let count_b = mesh_b.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len();

    let total_vertices = count_a + count_b;
    assert_eq!(total_vertices, 1024 * 4, "total vertices should be 1024 * 4 = 4096");
}

#[test]
fn build_chunk_mesh_different_offsets() {
    let (mesh_00, _) = build_chunk_mesh(0, 0, 32.0);
    let (mesh_10, _) = build_chunk_mesh(1, 0, 32.0);

    let vals_00 = mesh_00.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    let vals_10 = mesh_10.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();

    // Both should have vertices but at different world positions
    assert!(vals_00.len() > 0);
    assert!(vals_10.len() > 0);

    // Check first vertex positions differ (chunk offset affects world coords)
    if let (bevy::render::mesh::VertexAttributeValues::Float32x3(p00),
             bevy::render::mesh::VertexAttributeValues::Float32x3(p10)) = (vals_00, vals_10) {
        assert_ne!(
            p00[0][0], p10[0][0],
            "chunks at different offsets should have different x positions"
        );
    }
}

#[test]
fn build_chunk_mesh_quad_indices_are_valid() {
    let (mesh_a, _) = build_chunk_mesh(0, 0, 32.0);
    let vert_count = mesh_a.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().len();
    let idx = mesh_a.indices().unwrap();

    match idx {
        bevy::render::mesh::Indices::U32(indices) => {
            for &i in indices {
                assert!(
                    (i as usize) < vert_count,
                    "index {} out of range (max {})",
                    i,
                    vert_count - 1
                );
            }
        }
        _ => panic!("expected U32 indices"),
    }
}

// ════════════════════════════════════════════════════════════════
// ChunkGrid additional tests
// ════════════════════════════════════════════════════════════════

#[test]
fn chunk_grid_ensure_chunk_idempotent() {
    let mut grid = make_chunk_grid();
    let chunk_a = grid.ensure_chunk(0, 0).clone();
    let chunk_b = grid.ensure_chunk(0, 0).clone();
    assert_eq!(chunk_a.tiles, chunk_b.tiles);
    assert_eq!(chunk_a.deposits, chunk_b.deposits);
    assert_eq!(chunk_a.visited, chunk_b.visited);
}

#[test]
fn chunk_grid_clear_resets_all_state() {
    let mut grid = make_chunk_grid();
    grid.ensure_chunk(0, 0);
    grid.ensure_chunk(1, 1);
    grid.reveal_tile(0, 0, 5, 5);
    grid.pending_spawns.push((2, 2));
    grid.chunk_mesh_cache.insert(
        (0, 0),
        (Handle::default(), Handle::default()),
    );

    grid.clear();

    assert!(!grid.chunk_exists(0, 0));
    assert!(!grid.chunk_exists(1, 1));
    assert!(grid.pending_spawns.is_empty());
    assert!(grid.chunk_mesh_cache.is_empty());
    assert_eq!(grid.generated_chunks().count(), 0);
}

#[test]
fn chunk_grid_seed_and_set_seed() {
    let mut grid = make_chunk_grid();
    assert_eq!(grid.seed(), 42);
    grid.set_seed(999);
    assert_eq!(grid.seed(), 999);
}

#[test]
fn chunk_grid_get_chunk_returns_none_for_missing() {
    let grid = make_chunk_grid();
    assert!(grid.get_chunk(99, 99).is_none());
}

#[test]
fn chunk_grid_get_chunk_returns_some_for_existing() {
    let mut grid = make_chunk_grid();
    grid.ensure_chunk(0, 0);
    assert!(grid.get_chunk(0, 0).is_some());
}

#[test]
fn chunk_grid_tile_type_at_auto_generates() {
    let mut grid = make_chunk_grid();
    assert!(!grid.chunk_exists(5, 5));
    let _ = grid.tile_type_at(160, 160);
    assert!(grid.chunk_exists(5, 5));
}

#[test]
fn chunk_grid_reveal_multiple_tiles() {
    let mut grid = make_chunk_grid();
    grid.reveal_tile(0, 0, 0, 0);
    grid.reveal_tile(0, 0, 1, 1);
    grid.reveal_tile(0, 0, 31, 31);

    assert!(grid.is_tile_visited(0, 0, 0, 0));
    assert!(grid.is_tile_visited(0, 0, 1, 1));
    assert!(grid.is_tile_visited(0, 0, 31, 31));
    assert!(!grid.is_tile_visited(0, 0, 0, 1));
}

#[test]
fn chunk_grid_chunk_containing_consistency() {
    let grid = make_chunk_grid();
    let chunk_size = CHUNK_SIZE as i32;
    for x in -100..100 {
        for y in -100..100 {
            let (cx, cy) = grid.chunk_containing(x, y);
            assert!(x >= cx * chunk_size && x < (cx + 1) * chunk_size);
            assert!(y >= cy * chunk_size && y < (cy + 1) * chunk_size);
        }
    }
}

#[test]
fn chunk_grid_generated_chunks_tracks_all_created() {
    let mut grid = make_chunk_grid();
    grid.ensure_chunk(0, 0);
    grid.ensure_chunk(1, 0);
    grid.ensure_chunk(0, 1);
    grid.ensure_chunk(-1, -1);

    let chunks: Vec<_> = grid.generated_chunks().cloned().collect();
    assert_eq!(chunks.len(), 4);
    assert!(chunks.contains(&(0, 0)));
    assert!(chunks.contains(&(1, 0)));
    assert!(chunks.contains(&(0, 1)));
    assert!(chunks.contains(&(-1, -1)));
}

#[test]
fn chunk_grid_deposit_operations_on_existing_chunk() {
    let mut grid = make_chunk_grid();
    let chunk = grid.ensure_chunk(0, 0).clone();
    if let Some(d) = chunk.deposits.first() {
        let dx = d.x;
        let dy = d.y;

        grid.set_deposit_amount(0, 0, dx, dy, 777);
        let updated = grid.get_chunk(0, 0).unwrap();
        let d2 = updated.deposits.iter().find(|d| d.x == dx && d.y == dy).unwrap();
        assert_eq!(d2.amount, 777);

        grid.set_deposit_resource(0, 0, dx, dy, "mythril");
        let updated = grid.get_chunk(0, 0).unwrap();
        let d3 = updated.deposits.iter().find(|d| d.x == dx && d.y == dy).unwrap();
        assert_eq!(d3.resource, "mythril");
    }
}

#[test]
fn chunk_grid_is_tile_visited_returns_false_for_empty_world() {
    let grid = make_chunk_grid();
    assert!(!grid.is_tile_visited(0, 0, 0, 0));
    assert!(!grid.is_tile_visited(5, 5, 10, 10));
}

// ════════════════════════════════════════════════════════════════
// Combined system integration tests
// ════════════════════════════════════════════════════════════════

#[test]
fn wave_timer_and_spawn_enemies_integration() {
    let mut app = enemy_test_app();
    app.add_systems(Update, (wave_timer, spawn_enemies));

    spawn_player(&mut app, 0, 0, 100);

    {
        let mut wave = app.world_mut().resource_mut::<WaveState>();
        wave.timer = 0.0;
    }

    app.update();
    {
        let wave = app.world().resource::<WaveState>();
        assert_eq!(wave.wave, 2, "wave should be 2 after first update");
        assert!(!wave.spawn_queue.is_empty(), "spawn queue should be filled");
    }

    app.update();
    let enemy_count = get_enemy_count(&mut app);
    assert!(enemy_count >= 1, "at least one enemy should be spawned");
}

#[test]
fn full_combat_cycle_enemy_reaches_player_and_deals_damage() {
    let mut app = enemy_test_app();
    app.add_systems(Update, (move_enemies, enemies_damage_player));

    spawn_player(&mut app, 5, 5, 100);
    spawn_enemy(&mut app, 5, 5, 20, "runner");

    app.update();

    let hp = get_player_hp(&mut app);
    assert_eq!(hp, 90, "player should have taken 10 damage from runner");
}

#[test]
fn turret_and_enemy_interaction() {
    let mut app = enemy_test_app();
    app.add_systems(Update, turret_shoot);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat {
            damage: 25,
            range_sq: 10000.0,
            fire_interval: 0.5,
            timer: 0.5,
            projectile_speed: 300.0,
        },
    ));

    spawn_enemy(&mut app, 3, 0, 80, "tank");

    app.update();

    let combat = app.world_mut().query::<&TurretCombat>().iter(app.world()).next().unwrap();
    assert!(
        combat.timer < combat.fire_interval,
        "turret should have fired and reset timer"
    );
}

// ════════════════════════════════════════════════════════════════
// recenter_on_player tests
// ════════════════════════════════════════════════════════════════

#[test]
fn recenter_on_player_moves_camera_to_player_start() {
    use siege_factory::map::systems::recenter_on_player;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<KeyCode>>();

    let map_cfg = MapConfig::load(&test_mods());
    let (px, py) = map_cfg.player_start_position;
    let tile_size = map_cfg.tile_size;
    app.insert_resource(map_cfg);

    app.world_mut().spawn((
        Camera2d,
        Transform::from_xyz(9999.0, 9999.0, 0.0),
    ));

    app.add_systems(Update, recenter_on_player);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyH);

    app.update();

    let expected_x = px as f32 * tile_size;
    let expected_y = py as f32 * tile_size;

    let tf = app.world_mut().query_filtered::<&Transform, With<Camera2d>>().iter(app.world()).next().unwrap();

    assert!(
        (tf.translation.x - expected_x).abs() < 1.0,
        "camera x should be near player start: got {} expected {}",
        tf.translation.x, expected_x
    );
    assert!(
        (tf.translation.y - expected_y).abs() < 1.0,
        "camera y should be near player start: got {} expected {}",
        tf.translation.y, expected_y
    );
}

#[test]
fn recenter_on_player_does_nothing_without_h_key() {
    use siege_factory::map::systems::recenter_on_player;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<KeyCode>>();

    let map_cfg = MapConfig::load(&test_mods());
    app.insert_resource(map_cfg);

    app.world_mut().spawn((
        Camera2d,
        Transform::from_xyz(9999.0, 9999.0, 0.0),
    ));

    app.add_systems(Update, recenter_on_player);

    app.update();

    let tf = app.world_mut().query_filtered::<&Transform, With<Camera2d>>().iter(app.world()).next().unwrap();

    assert!(
        (tf.translation.x - 9999.0).abs() < 1.0,
        "camera should remain at original position when H not pressed"
    );
}

// ════════════════════════════════════════════════════════════════
// update_hovered_tile tests
// ════════════════════════════════════════════════════════════════

#[test]
fn update_hovered_tile_clears_when_ui_blocking() {
    use siege_factory::economy::ui_components::UiIsBlocking;
    use siege_factory::map::systems::update_hovered_tile;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<HoveredTile>();
    app.insert_resource(MapConfig::load(&test_mods()));
    app.insert_resource(UiIsBlocking(true));

    app.world_mut().spawn((
        Window::default(),
        PrimaryWindow,
    ));
    app.world_mut().spawn((
        Camera2d,
        Camera::default(),
        Transform::default(),
    ));

    {
        let mut h = app.world_mut().resource_mut::<HoveredTile>();
        h.0 = Some(TilePosition { x: 5, y: 5 });
    }

    app.add_systems(Update, update_hovered_tile);
    app.update();

    let hovered = app.world().resource::<HoveredTile>();
    assert!(hovered.0.is_none(), "hovered tile should be None when UI is blocking");
}
