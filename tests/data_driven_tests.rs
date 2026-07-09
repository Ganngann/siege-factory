use bevy::prelude::*;
use siege_factory::core::modding::ModRegistry;
use siege_factory::economy::belt::compute_slot_positions;
use siege_factory::economy::building::BuildingRegistry;
use siege_factory::economy::discovery::{DiscoveryRegistry, GlobalArchive};
use siege_factory::economy::game_components::Direction;
use siege_factory::economy::menu::{breadcrumb_at, format_cost, items_at, MenuAction, MenuDef, MenuEntry};
use siege_factory::economy::power::is_in_range;
use siege_factory::economy::recipe::RecipeRegistry;
use siege_factory::economy::resource::{Inventory, ResourceId, ResourceRegistry};
use siege_factory::economy::spatial::SpatialRegistry;
use siege_factory::economy::tiered_structure::ProgressionLogRegistry;
use siege_factory::economy::unit_config::UnitConfig;
use siege_factory::enemy::ai::bfs;
use siege_factory::enemy::registry::EnemyRegistry;
use siege_factory::enemy::wave_config::WaveConfig;
use siege_factory::map::tile_grid::ChunkGrid;
use siege_factory::rendering::config::VisualsConfig;
use std::collections::HashSet;

fn test_mods() -> ModRegistry {
    ModRegistry::for_test()
}

// ════════════════════════════════════════════════════════════════
// BuildingRegistry invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn all_buildings_have_positive_hp() {
    let reg = BuildingRegistry::load(&test_mods());
    for b in &reg.buildings {
        assert!(b.hp > 0, "building {} has hp 0", b.id);
    }
}

#[test]
fn all_buildings_have_nonempty_name() {
    let reg = BuildingRegistry::load(&test_mods());
    for b in &reg.buildings {
        assert!(!b.name.is_empty(), "building {} has empty name", b.id);
    }
}

#[test]
fn all_buildings_have_valid_tile_size() {
    let reg = BuildingRegistry::load(&test_mods());
    for b in &reg.buildings {
        assert!(b.tile_size.0 >= 1, "building {} has w=0", b.id);
        assert!(b.tile_size.1 >= 1, "building {} has h=0", b.id);
    }
}

#[test]
fn building_ids_are_unique() {
    let reg = BuildingRegistry::load(&test_mods());
    let mut ids = std::collections::HashSet::new();
    for b in &reg.buildings {
        assert!(ids.insert(&b.id), "Duplicate building id: {}", b.id);
    }
}

#[test]
fn building_registry_loads_nonempty() {
    let reg = BuildingRegistry::load(&test_mods());
    assert!(!reg.buildings.is_empty());
}

#[test]
fn building_registry_get_returns_none_for_unknown() {
    let reg = BuildingRegistry::load(&test_mods());
    assert!(reg.get("nonexistent_building_xyzzy").is_none());
}

#[test]
fn settings_refund_ratio_in_valid_range() {
    let settings = siege_factory::economy::building::DefaultSettings::load(&test_mods());
    assert!(settings.refund_ratio >= 0.0);
    assert!(settings.refund_ratio <= 1.0);
    assert!(settings.default_projectile_speed > 0.0);
}

// ════════════════════════════════════════════════════════════════
// RecipeRegistry invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn recipe_registry_loads_nonempty() {
    let reg = RecipeRegistry::load(&test_mods());
    assert!(!reg.recipes.is_empty());
}

#[test]
fn recipe_registry_get_returns_none_for_unknown() {
    let reg = RecipeRegistry::load(&test_mods());
    assert!(reg.get("nonexistent_recipe_xyzzy").is_none());
}

#[test]
fn all_recipes_have_positive_time() {
    let reg = RecipeRegistry::load(&test_mods());
    for recipe in reg.recipes.values() {
        assert!(recipe.time_sec > 0.0, "recipe {} has non-positive time", recipe.id);
    }
}

#[test]
fn all_recipes_have_nonempty_category() {
    let reg = RecipeRegistry::load(&test_mods());
    for recipe in reg.recipes.values() {
        assert!(!recipe.category.is_empty(), "recipe {} has empty category", recipe.id);
    }
}

#[test]
fn all_recipes_have_positive_output_amounts() {
    let reg = RecipeRegistry::load(&test_mods());
    for recipe in reg.recipes.values() {
        for (res, amt) in &recipe.output {
            assert!(*amt > 0, "recipe {} output {} has amount 0", recipe.id, res.0);
        }
    }
}

#[test]
fn all_recipes_have_lowercase_output_ids() {
    let reg = RecipeRegistry::load(&test_mods());
    for recipe in reg.recipes.values() {
        for (res, _) in &recipe.output {
            assert_eq!(res.0, res.0.to_lowercase(), "recipe {} output key not lowercase: {}", recipe.id, res.0);
        }
    }
}

#[test]
fn recipe_output_starts_with_nonzero() {
    let reg = RecipeRegistry::load(&test_mods());
    for recipe in reg.recipes.values() {
        assert!(
            !recipe.output.is_empty() || !recipe.fluid_output.is_empty(),
            "recipe {} has no output (neither items nor fluids)",
            recipe.id,
        );
    }
}

// ════════════════════════════════════════════════════════════════
// ResourceRegistry invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn resource_registry_loads_nonempty() {
    let reg = ResourceRegistry::load(&test_mods());
    assert!(!reg.resources.is_empty());
}

#[test]
fn all_resources_have_positive_max_stack() {
    let reg = ResourceRegistry::load(&test_mods());
    for def in reg.resources.values() {
        assert!(def.max_stack > 0, "{} has max_stack 0", def.id);
    }
}

#[test]
fn all_resources_have_nonempty_id() {
    let reg = ResourceRegistry::load(&test_mods());
    for (key, def) in &reg.resources {
        assert!(!key.is_empty());
        assert_eq!(key, &def.id);
    }
}

#[test]
fn resource_registry_get_opt_returns_none_for_unknown() {
    let reg = ResourceRegistry::load(&test_mods());
    assert!(reg.get_opt("unobtainium_xyzzy").is_none());
}

#[test]
fn resource_registry_known_keys_roundtrip() {
    let reg = ResourceRegistry::load(&test_mods());
    for (key, def) in &reg.resources {
        let retrieved = reg.get(key);
        assert_eq!(retrieved.id, def.id);
        assert_eq!(retrieved.name, def.name);
        assert_eq!(retrieved.max_stack, def.max_stack);
    }
}

// ════════════════════════════════════════════════════════════════
// UnitConfig invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn unit_config_loads_nonempty() {
    let cfg = UnitConfig::load(&test_mods());
    assert!(!cfg.units.is_empty());
}

#[test]
fn all_units_have_positive_hp() {
    let cfg = UnitConfig::load(&test_mods());
    for (id, def) in &cfg.units {
        assert!(def.hp > 0, "unit {} has hp 0", id);
    }
}

#[test]
fn all_units_have_nonempty_name() {
    let cfg = UnitConfig::load(&test_mods());
    for (id, def) in &cfg.units {
        assert!(!def.name.is_empty(), "unit {} has empty name", id);
    }
}

#[test]
fn unit_config_get_returns_none_for_unknown() {
    let cfg = UnitConfig::load(&test_mods());
    assert!(cfg.get("dragon_xyzzy").is_none());
}

// ════════════════════════════════════════════════════════════════
// EnemyRegistry invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn enemy_registry_loads_nonempty() {
    let reg = EnemyRegistry::load(&test_mods());
    assert!(!reg.enemies.is_empty());
}

#[test]
fn enemy_ids_are_unique() {
    let reg = EnemyRegistry::load(&test_mods());
    let mut ids = std::collections::HashSet::new();
    for id in reg.enemies.keys() {
        assert!(ids.insert(id.clone()), "duplicate enemy id: {}", id);
    }
}

#[test]
fn all_enemies_have_positive_hp() {
    let reg = EnemyRegistry::load(&test_mods());
    for def in reg.enemies.values() {
        assert!(def.hp > 0, "enemy {} has hp 0", def.id);
    }
}

#[test]
fn all_enemies_have_positive_speed() {
    let reg = EnemyRegistry::load(&test_mods());
    for def in reg.enemies.values() {
        assert!(def.speed > 0.0, "enemy {} has speed 0", def.id);
    }
}

#[test]
fn enemy_registry_get_returns_none_for_unknown() {
    let reg = EnemyRegistry::load(&test_mods());
    assert!(reg.get("nonexistent_enemy_xyzzy").is_none());
}

#[test]
fn all_enemies_have_nonempty_name() {
    let reg = EnemyRegistry::load(&test_mods());
    for def in reg.enemies.values() {
        assert!(!def.name.is_empty(), "enemy {} has empty name", def.id);
    }
}

// ════════════════════════════════════════════════════════════════
// WaveConfig invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn all_waves_have_valid_entries() {
    let cfg = WaveConfig::load(&test_mods());
    assert!(!cfg.waves.is_empty());
    for (i, wave) in cfg.waves.iter().enumerate() {
        assert!(!wave.enemies.is_empty(), "wave {} has no enemies", i + 1);
        for entry in &wave.enemies {
            assert!(!entry.kind.is_empty(), "wave {} has empty kind", i + 1);
            assert!(entry.count > 0, "wave {} entry {} has count 0", i + 1, entry.kind);
        }
    }
}

#[test]
fn wave_config_constants_positive() {
    let cfg = WaveConfig::load(&test_mods());
    assert!(cfg.win_waves > 0);
    assert!(cfg.first_wave_delay > 0.0);
    assert!(cfg.wave_interval_sec > 0.0);
    assert!(cfg.spawn_interval_sec > 0.0);
    assert!(cfg.spawn_timer_min > 0.0);
    assert!(cfg.projectile_hit_distance > 0.0);
    assert!(cfg.spawn_distance > 0.0);
    assert!(cfg.enemy_arrival_threshold > 0.0);
    assert!(cfg.max_enemies_base > 0);
    assert!(cfg.max_enemies_cap >= cfg.max_enemies_base);
}

// ════════════════════════════════════════════════════════════════
// DiscoveryRegistry invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn discovery_registry_loads_nonempty() {
    let reg = DiscoveryRegistry::load(&test_mods());
    assert!(!reg.discoveries.is_empty());
}

#[test]
fn discovery_registry_has_starter_recipes() {
    let reg = DiscoveryRegistry::load(&test_mods());
    assert!(!reg.starter_recipes.is_empty());
}

#[test]
fn all_discoveries_have_building() {
    let reg = DiscoveryRegistry::load(&test_mods());
    for d in &reg.discoveries {
        assert!(!d.building.is_empty(), "discovery {} has empty building", d.reward_id);
    }
}

#[test]
fn all_discoveries_have_positive_threshold() {
    let reg = DiscoveryRegistry::load(&test_mods());
    for d in &reg.discoveries {
        assert!(d.threshold > 0, "discovery {} has threshold 0", d.reward_id);
    }
}

// ════════════════════════════════════════════════════════════════
// VisualsConfig invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn visuals_config_positive_dimensions() {
    let cfg = VisualsConfig::load(&test_mods());
    assert!(cfg.hp_bar.width > 0.0);
    assert!(cfg.hp_bar.height > 0.0);
    assert!(cfg.belt_item.width > 0.0);
    assert!(cfg.belt_item.height > 0.0);
    assert!(cfg.unit.width > 0.0);
    assert!(cfg.unit.height > 0.0);
    assert!(cfg.player.width > 0.0);
    assert!(cfg.player.height > 0.0);
    assert!(cfg.builder.width > 0.0);
    assert!(cfg.builder.height > 0.0);
    assert!(cfg.projectile.scale > 0.0);
    assert!(cfg.mining_bar.width > 0.0);
    assert!(cfg.mining_bar.height > 0.0);
    assert!(cfg.toast.lifetime > 0.0);
    assert!(cfg.toast.font_size > 0.0);
    assert!(cfg.tile_highlight.alpha > 0.0);
    assert!(cfg.deposit_sprite.scale_ratio > 0.0);
    assert!(cfg.decoration.tree_z > 0.0);
    assert!(cfg.decoration.rock_z > 0.0);
    assert!(!cfg.decorations.is_empty());
}

#[test]
fn visuals_config_enemy_sizes_ordered() {
    let cfg = VisualsConfig::load(&test_mods());
    assert!(cfg.enemy.boss_size > cfg.enemy.tank_size);
    assert!(cfg.enemy.tank_size > cfg.enemy.default_size);
}

#[test]
fn visuals_config_decoration_entries_valid() {
    let cfg = VisualsConfig::load(&test_mods());
    for dec in &cfg.decorations {
        assert!(!dec.kind.is_empty());
        assert!(dec.min_size > 0.0);
        assert!(dec.max_size >= dec.min_size);
        assert!(dec.density >= 0.0);
    }
}

// ════════════════════════════════════════════════════════════════
// Inventory property-based tests
// ════════════════════════════════════════════════════════════════

#[test]
fn inventory_new_is_empty() {
    let inv = Inventory::new();
    assert_eq!(inv.total(), 0);
    assert!(!inv.is_full());
}

#[test]
fn inventory_add_then_remove_never_underflows() {
    let mut inv = Inventory::new();
    let rid = ResourceId::new("test_res");
    inv.add(&rid, 100);
    let before = inv.get(&rid);
    inv.remove(&rid, 50);
    assert!(inv.get(&rid) <= before);
    inv.remove(&rid, 50);
    assert_eq!(inv.get(&rid), 0);
    // remove from empty fails
    assert!(!inv.remove(&rid, 1));
}

#[test]
fn inventory_saturating_add() {
    let mut inv = Inventory::new();
    let rid = ResourceId::new("test_res");
    inv.add(&rid, u32::MAX);
    inv.add(&rid, 100);
    assert_eq!(inv.get(&rid), u32::MAX);
}

#[test]
fn inventory_capacity_limits() {
    let mut inv = Inventory::with_capacity(10);
    let rid = ResourceId::new("test_res");
    assert!(inv.try_add(&rid, 10));
    assert!(!inv.try_add(&rid, 1));
    assert_eq!(inv.get(&rid), 10);
}

#[test]
fn inventory_zero_capacity_no_limit() {
    let mut inv = Inventory::with_capacity(0);
    let rid = ResourceId::new("test_res");
    assert!(inv.try_add(&rid, 1_000_000));
    assert_eq!(inv.get(&rid), 1_000_000);
}

#[test]
fn inventory_total_matches_sum() {
    let mut inv = Inventory::new();
    let a = ResourceId::new("res_a");
    let b = ResourceId::new("res_b");
    inv.add(&a, 10);
    inv.add(&b, 25);
    assert_eq!(inv.total(), 35);
}

#[test]
fn inventory_different_resources_independent() {
    let mut inv = Inventory::new();
    let a = ResourceId::new("res_a");
    let b = ResourceId::new("res_b");
    inv.add(&a, 10);
    inv.add(&b, 5);
    assert_eq!(inv.get(&a), 10);
    assert_eq!(inv.get(&b), 5);
    inv.remove(&a, 10);
    assert_eq!(inv.get(&a), 0);
    assert_eq!(inv.get(&b), 5);
}

proptest::proptest! {
    #[test]
    fn inventory_never_negative(
        add1 in 0..1000u32,
        add2 in 0..1000u32,
        remove in 0..2000u32,
    ) {
        let rid = ResourceId::new("proptest_res");
        let mut inv = Inventory::new();
        inv.add(&rid, add1);
        inv.add(&rid, add2);
        let before = inv.get(&rid);
        inv.remove(&rid, remove);
        assert!(inv.get(&rid) <= before);
    }
}

// ════════════════════════════════════════════════════════════════
// BFS property tests
// ════════════════════════════════════════════════════════════════

#[test]
fn bfs_start_equals_goal() {
    let blocked = HashSet::new();
    let result = bfs((3, 3), (3, 3), &blocked, 1000);
    assert_eq!(result, Some(vec![]));
}

#[test]
fn bfs_start_blocked_returns_none() {
    let mut blocked = HashSet::new();
    blocked.insert((0, 0));
    assert!(bfs((0, 0), (5, 5), &blocked, 1000).is_none());
}

#[test]
fn bfs_adjacent_goal() {
    let blocked = HashSet::new();
    let path = bfs((0, 0), (1, 0), &blocked, 1000).unwrap();
    assert_eq!(path, vec![(1, 0)]);
}

#[test]
fn bfs_path_reaches_goal() {
    let blocked = HashSet::new();
    let path = bfs((0, 0), (5, 5), &blocked, 1000).unwrap();
    assert_eq!(*path.last().unwrap(), (5, 5));
}

#[test]
fn bfs_each_step_moves_one_tile() {
    let blocked = HashSet::new();
    let path = bfs((0, 0), (5, 5), &blocked, 1000).unwrap();
    let mut prev = (0, 0);
    for step in &path {
        let dx = (step.0 - prev.0).abs();
        let dy = (step.1 - prev.1).abs();
        assert_eq!(dx + dy, 1, "step {:?} -> {:?} is not adjacent", prev, step);
        prev = *step;
    }
}

#[test]
fn bfs_path_avoids_blocked() {
    let mut blocked = HashSet::new();
    blocked.insert((1, 0));
    blocked.insert((1, 1));
    let path = bfs((0, 0), (2, 0), &blocked, 1000).unwrap();
    for step in &path {
        assert!(!blocked.contains(step), "path went through blocked tile {:?}", step);
    }
    assert_eq!(*path.last().unwrap(), (2, 0));
}

#[test]
fn bfs_4_directional() {
    let blocked = HashSet::new();
    assert_eq!(bfs((0, 0), (0, 1), &blocked, 1000).unwrap(), vec![(0, 1)]);
    assert_eq!(bfs((0, 0), (-1, 0), &blocked, 1000).unwrap(), vec![(-1, 0)]);
    assert_eq!(bfs((0, 0), (0, -1), &blocked, 1000).unwrap(), vec![(0, -1)]);
    assert_eq!(bfs((0, 0), (1, 0), &blocked, 1000).unwrap(), vec![(1, 0)]);
}

#[test]
fn bfs_negative_coords() {
    let blocked = HashSet::new();
    let path = bfs((-5, -5), (-3, -5), &blocked, 1000).unwrap();
    assert_eq!(path, vec![(-4, -5), (-3, -5)]);
}

#[test]
fn bfs_max_nodes_limit() {
    let blocked = HashSet::new();
    assert!(bfs((0, 0), (100, 100), &blocked, 5).is_none());
}

// ════════════════════════════════════════════════════════════════
// ChunkGrid determinism & invariants
// ════════════════════════════════════════════════════════════════

fn test_dist() -> Vec<(String, u32)> {
    vec![
        ("iron_ore".to_string(), 50),
        ("copper_ore".to_string(), 35),
        ("coal".to_string(), 15),
    ]
}

#[test]
fn chunk_grid_same_seed_same_chunk() {
    let dist = test_dist();
    let mut a = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist.clone());
    let mut b = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    assert_eq!(
        a.ensure_chunk(0, 0).tiles,
        b.ensure_chunk(0, 0).tiles
    );
    assert_eq!(
        a.ensure_chunk(0, 0).deposits,
        b.ensure_chunk(0, 0).deposits
    );
}

#[test]
fn chunk_grid_deterministic_across_chunks() {
    let dist = test_dist();
    let mut g1 = ChunkGrid::new(777, 50, 150, 35, 2, 5, dist.clone());
    let mut g2 = ChunkGrid::new(777, 50, 150, 35, 2, 5, dist);
    for cx in -2..=2 {
        for cy in -2..=2 {
            let a = g1.ensure_chunk(cx, cy).clone();
            let b = g2.ensure_chunk(cx, cy).clone();
            assert_eq!(a.tiles, b.tiles);
            assert_eq!(a.deposits, b.deposits);
        }
    }
}

#[test]
fn chunk_grid_different_seeds_different() {
    let dist = vec![("iron_ore".to_string(), 100)]; // 100% spawn
    let mut a = ChunkGrid::new(1, 50, 150, 100, 2, 5, dist.clone());
    let mut b = ChunkGrid::new(2, 50, 150, 100, 2, 5, dist);
    assert_ne!(a.ensure_chunk(0, 0).deposits, b.ensure_chunk(0, 0).deposits);
}

#[test]
fn chunk_grid_seed_accessor() {
    let dist = test_dist();
    let mut grid = ChunkGrid::new(12345, 50, 150, 35, 2, 5, dist);
    assert_eq!(grid.seed(), 12345);
    grid.set_seed(999);
    assert_eq!(grid.seed(), 999);
}

#[test]
fn chunk_grid_ensure_chunk_is_idempotent() {
    let dist = test_dist();
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    let a = grid.ensure_chunk(3, 4).clone();
    let b = grid.ensure_chunk(3, 4).clone();
    assert_eq!(a.tiles, b.tiles);
    assert_eq!(a.deposits, b.deposits);
}

#[test]
fn chunk_grid_clear_empties() {
    let dist = test_dist();
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    grid.ensure_chunk(0, 0);
    grid.clear();
    assert_eq!(grid.generated_chunks().count(), 0);
}

#[test]
fn chunk_grid_deposits_within_bounds() {
    let dist = test_dist();
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    let chunk = grid.ensure_chunk(0, 0);
    for d in &chunk.deposits {
        assert!(d.x < 32);
        assert!(d.y < 32);
        assert!(d.amount >= 50);
        assert!(d.amount <= 150);
    }
}

#[test]
fn chunk_grid_reveal_roundtrip() {
    let dist = test_dist();
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    assert!(!grid.is_tile_visited(0, 0, 5, 10));
    grid.reveal_tile(0, 0, 5, 10);
    assert!(grid.is_tile_visited(0, 0, 5, 10));
}

#[test]
fn chunk_grid_reveal_auto_generates() {
    let dist = test_dist();
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    assert!(!grid.chunk_exists(5, 5));
    grid.reveal_tile(5, 5, 0, 0);
    assert!(grid.chunk_exists(5, 5));
}

#[test]
fn chunk_grid_tile_type_always_valid() {
    let dist = test_dist();
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    let tt = grid.tile_type_at(100, 200);
    assert!(tt == siege_factory::map::components::TileType::Ground
        || tt == siege_factory::map::components::TileType::Resource);
}

#[test]
fn chunk_grid_deposit_distribution_respected() {
    let dist = vec![("iron_ore".to_string(), 100)];
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
    for cx in 0..5 {
        for cy in 0..5 {
            for d in &grid.ensure_chunk(cx, cy).deposits {
                assert_eq!(d.resource, "iron_ore");
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Belt slot property tests
// ════════════════════════════════════════════════════════════════

const TILE: f32 = 32.0;

#[test]
fn belt_slot_count_matches() {
    for num in 1..=8 {
        let positions = compute_slot_positions(0, 0, Direction::East, num, TILE);
        assert_eq!(positions.len(), num as usize, "num_slots={}", num);
    }
}

#[test]
fn belt_zero_slots_empty() {
    let positions = compute_slot_positions(0, 0, Direction::East, 0, TILE);
    assert!(positions.is_empty());
}

#[test]
fn belt_single_slot_at_center() {
    let positions = compute_slot_positions(0, 0, Direction::East, 1, TILE);
    assert!((positions[0] - Vec2::new(0.0, 0.0)).length() < 0.01);
}

#[test]
fn belt_slots_within_tile_bounds() {
    for dir in &Direction::ALL {
        let positions = compute_slot_positions(0, 0, *dir, 4, TILE);
        for pos in &positions {
            assert!(pos.x.abs() <= TILE / 2.0 + 0.01, "slot x {} out of bounds", pos.x);
            assert!(pos.y.abs() <= TILE / 2.0 + 0.01, "slot y {} out of bounds", pos.y);
        }
    }
}

#[test]
fn belt_slots_symmetrically_centered() {
    for n in 1..=8 {
        let positions = compute_slot_positions(0, 0, Direction::East, n, TILE);
        let com = positions.iter().fold(Vec2::ZERO, |acc, p| acc + *p) / n as f32;
        assert!(com.x.abs() < 0.01, "com x should be ~0 for n={}", n);
        assert!(com.y.abs() < 0.01, "com y should be ~0 for n={}", n);
    }
}

#[test]
fn belt_slot_spacing_uniform() {
    for n in 2..=8 {
        let positions = compute_slot_positions(0, 0, Direction::East, n, TILE);
        let spacing01 = (positions[1] - positions[0]).length();
        for i in 1..positions.len() {
            let spacing = (positions[i] - positions[i - 1]).length();
            assert!((spacing - spacing01).abs() < 0.01,
                "n={} spacing[{}]={} != spacing01={}", n, i, spacing, spacing01);
        }
    }
}

#[test]
fn belt_east_west_mirrored() {
    let east = compute_slot_positions(0, 0, Direction::East, 4, TILE);
    let west = compute_slot_positions(0, 0, Direction::West, 4, TILE);
    for i in 0..4 {
        assert!((east[i].x + west[i].x).abs() < 0.01);
    }
}

#[test]
fn belt_north_south_mirrored() {
    let north = compute_slot_positions(0, 0, Direction::North, 4, TILE);
    let south = compute_slot_positions(0, 0, Direction::South, 4, TILE);
    for i in 0..4 {
        assert!((north[i].y + south[i].y).abs() < 0.01);
    }
}

#[test]
fn belt_slot_positions_at_origin() {
    let positions = compute_slot_positions(5, 3, Direction::East, 2, TILE);
    let center = siege_factory::core::utils::tile_to_world(5, 3, TILE);
    let mid = (positions[0] + positions[1]) / 2.0;
    assert!((mid - center).length() < 0.01);
}

// ════════════════════════════════════════════════════════════════
// Power is_in_range property tests
// ════════════════════════════════════════════════════════════════

#[test]
fn is_in_range_within_distance() {
    let poles = vec![(Entity::PLACEHOLDER, Vec3::new(0.0, 0.0, 0.0), 10.0)];
    assert!(is_in_range(Vec3::new(5.0, 0.0, 0.0), &poles));
    assert!(is_in_range(Vec3::new(0.0, 10.0, 0.0), &poles));
    assert!(is_in_range(Vec3::new(-3.0, 4.0, 0.0), &poles));
}

#[test]
fn is_in_range_outside_distance() {
    let poles = vec![(Entity::PLACEHOLDER, Vec3::new(0.0, 0.0, 0.0), 5.0)];
    assert!(!is_in_range(Vec3::new(10.0, 0.0, 0.0), &poles));
    assert!(!is_in_range(Vec3::new(0.0, 10.0, 0.0), &poles));
}

#[test]
fn is_in_range_exact_boundary() {
    let poles = vec![(Entity::PLACEHOLDER, Vec3::new(0.0, 0.0, 0.0), 5.0)];
    assert!(is_in_range(Vec3::new(5.0, 0.0, 0.0), &poles));
}

#[test]
fn is_in_range_empty_poles() {
    assert!(!is_in_range(Vec3::new(0.0, 0.0, 0.0), &[]));
}

#[test]
fn is_in_range_multiple_poles() {
    let poles = vec![
        (Entity::PLACEHOLDER, Vec3::new(0.0, 0.0, 0.0), 5.0),
        (Entity::PLACEHOLDER, Vec3::new(100.0, 0.0, 0.0), 5.0),
    ];
    assert!(is_in_range(Vec3::new(3.0, 0.0, 0.0), &poles));
    assert!(is_in_range(Vec3::new(103.0, 0.0, 0.0), &poles));
    assert!(!is_in_range(Vec3::new(50.0, 0.0, 0.0), &poles));
}

// ════════════════════════════════════════════════════════════════
// SpatialRegistry tests
// ════════════════════════════════════════════════════════════════

fn e(id: u32) -> Entity {
    Entity::from_raw_u32(id).expect("valid entity index")
}

#[test]
fn spatial_at_returns_existing() {
    let mut reg = SpatialRegistry::default();
    reg.insert_for_test(0, 0, e(1));
    assert_eq!(reg.at(0, 0), Some(e(1)));
}

#[test]
fn spatial_at_returns_none_for_empty() {
    let reg = SpatialRegistry::default();
    assert_eq!(reg.at(5, 5), None);
}

#[test]
fn spatial_at_negative_coords() {
    let mut reg = SpatialRegistry::default();
    reg.insert_for_test(-3, -7, e(2));
    assert_eq!(reg.at(-3, -7), Some(e(2)));
    assert_eq!(reg.at(-3, -6), None);
}

#[test]
fn spatial_is_free() {
    let mut reg = SpatialRegistry::default();
    assert!(reg.is_free(0, 0));
    reg.insert_for_test(2, 3, e(1));
    assert!(!reg.is_free(2, 3));
    assert!(reg.is_free(2, 4));
}

#[test]
fn spatial_entities_in_rect() {
    let mut reg = SpatialRegistry::default();
    reg.insert_for_test(0, 0, e(1));
    reg.insert_for_test(1, 0, e(2));
    reg.insert_for_test(0, 1, e(3));
    let result = reg.entities_in_rect(0, 0, 1, 1);
    assert_eq!(result.len(), 3);
    assert!(result.contains(&e(1)));
    assert!(result.contains(&e(2)));
    assert!(result.contains(&e(3)));
}

#[test]
fn spatial_entities_in_rect_deduplicates() {
    let mut reg = SpatialRegistry::default();
    reg.insert_for_test(0, 0, e(1));
    reg.insert_for_test(1, 0, e(1));
    let result = reg.entities_in_rect(0, 0, 1, 0);
    assert_eq!(result.len(), 1);
}

#[test]
fn spatial_entities_in_rect_empty() {
    let mut reg = SpatialRegistry::default();
    reg.insert_for_test(0, 0, e(1));
    assert!(reg.entities_in_rect(5, 5, 5, 5).is_empty());
}

#[test]
fn spatial_tiles_are_free() {
    let reg = SpatialRegistry::default();
    assert!(reg.tiles_are_free(&[]));
    assert!(reg.tiles_are_free(&[(1, 1), (2, 2)]));
}

#[test]
fn spatial_occupied_tiles() {
    let mut reg = SpatialRegistry::default();
    assert_eq!(reg.occupied_tiles().count(), 0);
    reg.insert_for_test(0, 0, e(1));
    reg.insert_for_test(1, 2, e(2));
    assert_eq!(reg.occupied_tiles().count(), 2);
}

#[test]
fn spatial_default_is_empty() {
    let reg = SpatialRegistry::default();
    assert_eq!(reg.occupied_tiles().count(), 0);
}

// ════════════════════════════════════════════════════════════════
// ProgressionLogRegistry tests
// ════════════════════════════════════════════════════════════════

#[test]
fn progression_log_registry_default() {
    let reg = ProgressionLogRegistry::default();
    assert!(reg.logs.is_empty());
    assert!(reg.unlocked.is_empty());
}

#[test]
fn progression_log_unlock_returns_entry() {
    let mut reg = ProgressionLogRegistry::default();
    reg.logs.push(siege_factory::economy::tiered_structure::LogEntry {
        id: "log1".into(),
        tier: 1,
        title: "First".into(),
        text: "Text".into(),
    });
    let entry = reg.unlock("log1").expect("should return entry");
    assert_eq!(entry.id, "log1");
    assert_eq!(entry.tier, 1);
}

#[test]
fn progression_log_double_unlock_returns_none() {
    let mut reg = ProgressionLogRegistry::default();
    reg.logs.push(siege_factory::economy::tiered_structure::LogEntry {
        id: "log1".into(),
        tier: 1,
        title: "First".into(),
        text: "Text".into(),
    });
    assert!(reg.unlock("log1").is_some());
    assert!(reg.unlock("log1").is_none());
}

#[test]
fn progression_log_unlock_unknown_returns_none() {
    let mut reg = ProgressionLogRegistry::default();
    assert!(reg.unlock("nonexistent").is_none());
}

#[test]
fn progression_log_unlock_multiple() {
    let mut reg = ProgressionLogRegistry::default();
    for i in 0..3 {
        reg.logs.push(siege_factory::economy::tiered_structure::LogEntry {
            id: format!("log{}", i),
            tier: i + 1,
            title: format!("Title {}", i),
            text: format!("Text {}", i),
        });
    }
    assert!(reg.unlock("log0").is_some());
    assert!(reg.unlock("log1").is_some());
    assert!(reg.unlock("log2").is_some());
    assert_eq!(reg.unlocked.len(), 3);
}

#[test]
fn progression_log_unlock_is_idempotent_in_set() {
    let mut reg = ProgressionLogRegistry::default();
    reg.logs.push(siege_factory::economy::tiered_structure::LogEntry {
        id: "log1".into(),
        tier: 1,
        title: "First".into(),
        text: "Text".into(),
    });
    reg.unlock("log1");
    assert!(reg.unlocked.contains("log1"));
}

// ════════════════════════════════════════════════════════════════
// GlobalArchive tests
// ════════════════════════════════════════════════════════════════

#[test]
fn global_archive_empty_starters() {
    let archive = GlobalArchive::new(&[]);
    assert!(!archive.is_unlocked("anything"));
}

#[test]
fn global_archive_deduplicates() {
    let starters = vec!["a".into(), "a".into(), "b".into()];
    let archive = GlobalArchive::new(&starters);
    assert!(archive.is_unlocked("a"));
    assert!(archive.is_unlocked("b"));
}

#[test]
fn global_archive_starter_recipes_from_toml() {
    let disc_reg = DiscoveryRegistry::load(&test_mods());
    let archive = GlobalArchive::new(&disc_reg.starter_recipes);
    for recipe in &disc_reg.starter_recipes {
        assert!(archive.is_unlocked(recipe), "starter recipe {} not unlocked", recipe);
    }
}

// ════════════════════════════════════════════════════════════════
// format_cost tests
// ════════════════════════════════════════════════════════════════

#[test]
fn format_cost_empty() {
    assert_eq!(format_cost(&[]), "");
}

#[test]
fn format_cost_single_item() {
    use siege_factory::economy::resource::Cost;
    let cost = vec![Cost {
        resource: ResourceId::new("test_res"),
        amount: 10,
    }];
    let s = format_cost(&cost);
    assert!(s.contains("10"));
}

#[test]
fn format_cost_multiple_items() {
    use siege_factory::economy::resource::Cost;
    let cost = vec![
        Cost { resource: ResourceId::new("res_a"), amount: 5 },
        Cost { resource: ResourceId::new("res_b"), amount: 3 },
    ];
    let s = format_cost(&cost);
    assert!(s.contains('+'));
}

// ════════════════════════════════════════════════════════════════
// Menu navigation tests
// ════════════════════════════════════════════════════════════════

fn make_action(label: &str, id: &str) -> MenuEntry {
    MenuEntry::Action {
        label: label.to_string(),
        action: MenuAction::Build(id.to_string()),
    }
}

fn make_submenu(label: &str, items: Vec<MenuEntry>) -> MenuEntry {
    MenuEntry::SubMenu {
        label: label.to_string(),
        items,
    }
}

#[test]
fn items_at_root_returns_all() {
    let entries = vec![make_action("A", "a"), make_action("B", "b")];
    assert_eq!(items_at(&entries, &[]).len(), 2);
}

#[test]
fn items_at_empty_stack() {
    assert!(items_at(&[], &[]).is_empty());
}

#[test]
fn items_at_descends_into_submenu() {
    let sub = make_submenu("Sub", vec![make_action("X", "x")]);
    let entries = vec![make_action("A", "a"), sub];
    let result = items_at(&entries, &[1]);
    assert_eq!(result.len(), 1);
    match &result[0] {
        MenuEntry::Action { label, .. } => assert_eq!(label, "X"),
        _ => panic!("expected Action"),
    }
}

#[test]
fn items_at_action_returns_empty() {
    let entries = vec![make_action("A", "a")];
    assert!(items_at(&entries, &[0]).is_empty());
}

#[test]
fn breadcrumb_at_root_is_empty() {
    let entries = vec![make_action("A", "a")];
    assert_eq!(breadcrumb_at(&entries, &[]), "");
}

#[test]
fn breadcrumb_at_submenu() {
    let sub = make_submenu("Logistics", vec![make_action("Belt", "belt")]);
    let entries = vec![make_submenu("Production", vec![]), sub];
    assert_eq!(breadcrumb_at(&entries, &[1]), "Logistics");
}

#[test]
fn breadcrumb_at_nested() {
    let inner = make_submenu("Inner", vec![make_action("X", "x")]);
    let outer = make_submenu("Outer", vec![inner]);
    assert_eq!(breadcrumb_at(&[outer], &[0, 0]), "Outer > Inner");
}

#[test]
fn menu_action_equality() {
    assert_eq!(MenuAction::Build("a".into()), MenuAction::Build("a".into()));
    assert_ne!(MenuAction::Build("a".into()), MenuAction::Spawn("a".into()));
    assert_eq!(MenuAction::Delete, MenuAction::Delete);
}

// ════════════════════════════════════════════════════════════════
// MenuDef invariants
// ════════════════════════════════════════════════════════════════

#[test]
fn menu_def_root_not_empty() {
    let reg = BuildingRegistry::load(&test_mods());
    let unit_cfg = UnitConfig::load(&test_mods());
    let menu = MenuDef::load(&test_mods(), &reg, &unit_cfg);
    assert!(!menu.root.is_empty());
}

#[test]
fn menu_def_page_size_positive() {
    let reg = BuildingRegistry::load(&test_mods());
    let unit_cfg = UnitConfig::load(&test_mods());
    let menu = MenuDef::load(&test_mods(), &reg, &unit_cfg);
    assert!(menu.page_size > 0);
}

#[test]
fn menu_def_root_entries_are_submenus() {
    let reg = BuildingRegistry::load(&test_mods());
    let unit_cfg = UnitConfig::load(&test_mods());
    let menu = MenuDef::load(&test_mods(), &reg, &unit_cfg);
    for entry in &menu.root {
        match entry {
            MenuEntry::SubMenu { .. } => {}
            _ => panic!("root entry should be SubMenu"),
        }
    }
}

#[test]
fn menu_def_first_category_has_items() {
    let reg = BuildingRegistry::load(&test_mods());
    let unit_cfg = UnitConfig::load(&test_mods());
    let menu = MenuDef::load(&test_mods(), &reg, &unit_cfg);
    if let Some(MenuEntry::SubMenu { items, .. }) = menu.root.first() {
        assert!(!items.is_empty());
    }
}

#[test]
fn menu_def_navigation_roundtrip() {
    let reg = BuildingRegistry::load(&test_mods());
    let unit_cfg = UnitConfig::load(&test_mods());
    let menu = MenuDef::load(&test_mods(), &reg, &unit_cfg);
    let result = items_at(&menu.root, &[0, 0]);
    if let MenuEntry::SubMenu { items, .. } = &menu.root[0] {
        if let MenuEntry::SubMenu { items: inner_items, .. } = &items[0] {
            assert_eq!(result.len(), inner_items.len());
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Production timer logic (pure function)
// ════════════════════════════════════════════════════════════════

#[test]
fn production_timer_cycles_correctly() {
    let mut timer = 0.0_f32;
    let interval = 0.5_f32;
    let mut events = 0;
    for _ in 0..50 {
        timer += 0.1_f32;
        while timer >= interval {
            timer -= interval;
            events += 1;
        }
    }
    assert_eq!(events, 10);
    assert!((timer - 0.0).abs() < f32::EPSILON);
}

// ════════════════════════════════════════════════════════════════
// KeyBindings property tests
// ════════════════════════════════════════════════════════════════

#[test]
fn keybindings_all_returns_sorted() {
    let bindings = siege_factory::core::input::KeyBindings::load(&test_mods());
    let all = bindings.all();
    for pair in all.windows(2) {
        assert!(pair[0].0 <= pair[1].0);
    }
}

#[test]
fn keybindings_set_and_get() {
    let mut bindings = siege_factory::core::input::KeyBindings::load(&test_mods());
    bindings.set("test_action", siege_factory::core::input::InputBinding::Key(KeyCode::KeyX));
    let retrieved = bindings.get("test_action");
    assert!(matches!(retrieved, siege_factory::core::input::InputBinding::Key(KeyCode::KeyX)));
}

#[test]
fn keybindings_apply_overrides() {
    let mut bindings = siege_factory::core::input::KeyBindings::load(&test_mods());
    let mut overrides = std::collections::HashMap::new();
    overrides.insert("cancel".to_string(), "KeyQ".to_string());
    bindings.apply_overrides(&overrides);
    let updated = bindings.get("cancel");
    assert!(matches!(updated, siege_factory::core::input::InputBinding::Key(KeyCode::KeyQ)));
}

#[test]
fn keybindings_apply_overrides_ignores_invalid() {
    let mut bindings = siege_factory::core::input::KeyBindings::load(&test_mods());
    let original = format!("{}", bindings.get("cancel"));
    let mut overrides = std::collections::HashMap::new();
    overrides.insert("cancel".to_string(), "TotallyInvalid".to_string());
    bindings.apply_overrides(&overrides);
    assert_eq!(format!("{}", bindings.get("cancel")), original);
}

#[test]
fn keybindings_key_method() {
    let bindings = siege_factory::core::input::KeyBindings::load(&test_mods());
    let k = bindings.key("cancel");
    assert_eq!(k, KeyCode::Escape);
}

#[test]
fn keybindings_mouse_method() {
    let bindings = siege_factory::core::input::KeyBindings::load(&test_mods());
    let m = bindings.mouse("place");
    assert_eq!(m, MouseButton::Left);
}

// ════════════════════════════════════════════════════════════════
// Input binding parsing property tests
// ════════════════════════════════════════════════════════════════

#[test]
fn parse_input_binding_key() {
    let b = siege_factory::core::input::parse_input_binding("KeyA");
    assert!(matches!(b, Some(siege_factory::core::input::InputBinding::Key(KeyCode::KeyA))));
}

#[test]
fn parse_input_binding_mouse() {
    let b = siege_factory::core::input::parse_input_binding("MouseLeft");
    assert!(matches!(b, Some(siege_factory::core::input::InputBinding::Mouse(MouseButton::Left))));
}

#[test]
fn parse_input_binding_invalid() {
    assert!(siege_factory::core::input::parse_input_binding("Invalid").is_none());
    assert!(siege_factory::core::input::parse_input_binding("").is_none());
}

#[test]
fn parse_key_code_special_keys() {
    use bevy::input::keyboard::KeyCode;
    assert_eq!(siege_factory::core::input::parse_key_code("Space"), Some(KeyCode::Space));
    assert_eq!(siege_factory::core::input::parse_key_code("Escape"), Some(KeyCode::Escape));
    assert_eq!(siege_factory::core::input::parse_key_code("Enter"), Some(KeyCode::Enter));
    assert_eq!(siege_factory::core::input::parse_key_code("Tab"), Some(KeyCode::Tab));
    assert_eq!(siege_factory::core::input::parse_key_code("ArrowUp"), Some(KeyCode::ArrowUp));
    assert_eq!(siege_factory::core::input::parse_key_code("ArrowDown"), Some(KeyCode::ArrowDown));
}

#[test]
fn parse_key_code_invalid() {
    assert_eq!(siege_factory::core::input::parse_key_code("NotAKey"), None);
    assert_eq!(siege_factory::core::input::parse_key_code(""), None);
    assert_eq!(siege_factory::core::input::parse_key_code("keya"), None);
    assert_eq!(siege_factory::core::input::parse_key_code("F99"), None);
}

#[test]
fn parse_mouse_button_valid() {
    assert_eq!(siege_factory::core::input::parse_mouse_button("MouseLeft"), Some(MouseButton::Left));
    assert_eq!(siege_factory::core::input::parse_mouse_button("MouseRight"), Some(MouseButton::Right));
    assert_eq!(siege_factory::core::input::parse_mouse_button("MouseMiddle"), Some(MouseButton::Middle));
}

#[test]
fn parse_mouse_button_invalid() {
    assert_eq!(siege_factory::core::input::parse_mouse_button("MouseBack"), None);
    assert_eq!(siege_factory::core::input::parse_mouse_button(""), None);
    assert_eq!(siege_factory::core::input::parse_mouse_button("Left"), None);
}
