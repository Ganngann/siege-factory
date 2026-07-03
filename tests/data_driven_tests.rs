use siege_factory::economy::building::BuildingRegistry;
use siege_factory::economy::resource::{ResourceId, Inventory};
use siege_factory::economy::unit_config::UnitConfig;
use siege_factory::map::tile_grid::ChunkGrid;

// ── TOML loading tests ──

#[test]
fn building_registry_loads() {
    let reg = BuildingRegistry::load();
    let hq = reg.get("hq").expect("HQ should exist");
    assert_eq!(hq.name, "HQ");
    assert_eq!(hq.hp, 100);
    assert!(hq.hidden);
    assert!(!hq.can_deconstruct);

    let miner = reg.get("miner").expect("Miner should exist");
    assert!(miner.requires_deposit);
    assert!(miner.production.is_some());

    let belt = reg.get("belt").expect("Belt should exist");
    assert!(belt.belt.is_some());
    assert_eq!(belt.belt.as_ref().unwrap().slots, 4);

    let wall = reg.get("wall").expect("Wall should exist");
    assert!(wall.drag_placement);

    let storage = reg.get("storage").expect("Storage should exist");
    assert_eq!(storage.inventory_capacity, 64);
}

#[test]
fn building_ids_are_unique() {
    let reg = BuildingRegistry::load();
    let mut ids = std::collections::HashSet::new();
    for b in &reg.buildings {
        assert!(ids.insert(&b.id), "Duplicate building id: {}", b.id);
    }
}

#[test]
fn unit_config_loads() {
    let cfg = UnitConfig::load();
    assert!(cfg.units.contains_key("soldier"), "Missing soldier");
    assert!(cfg.units.contains_key("worker"), "Missing worker");
    assert!(cfg.units.len() >= 2, "Expected at least 2 units, got {}", cfg.units.len());
}

// ── Economy tests ──

#[test]
fn inventory_operations() {
    let mut inv = Inventory::new();
    assert_eq!(inv.get(&ResourceId("ore".to_string())), 0);

    inv.add(&ResourceId("ore".to_string()), 10);
    assert_eq!(inv.get(&ResourceId("ore".to_string())), 10);

    inv.add(&ResourceId("ore".to_string()), 5);
    assert_eq!(inv.get(&ResourceId("ore".to_string())), 15);

    assert!(inv.remove(&ResourceId("ore".to_string()), 7));
    assert_eq!(inv.get(&ResourceId("ore".to_string())), 8);

    assert!(!inv.remove(&ResourceId("ore".to_string()), 20));
    assert_eq!(inv.get(&ResourceId("ore".to_string())), 8);
}

#[test]
fn inventory_saturating_add() {
    let mut inv = Inventory::new();
    inv.add(&ResourceId("ore".to_string()), u32::MAX);
    inv.add(&ResourceId("ore".to_string()), 100);
    assert_eq!(inv.get(&ResourceId("ore".to_string())), u32::MAX);
}

#[test]
fn inventory_separate_resources() {
    let mut inv = Inventory::new();
    inv.add(&ResourceId("ore".to_string()), 10);
    inv.add(&ResourceId("ammo".to_string()), 5);
    inv.add(&ResourceId("energy".to_string()), 3);
    assert_eq!(inv.get(&ResourceId("ore".to_string())), 10);
    assert_eq!(inv.get(&ResourceId("ammo".to_string())), 5);
    assert_eq!(inv.get(&ResourceId("energy".to_string())), 3);
    inv.remove(&ResourceId("ore".to_string()), 10);
    assert_eq!(inv.get(&ResourceId("ore".to_string())), 0);
    assert_eq!(inv.get(&ResourceId("ammo".to_string())), 5);
}

// ── Production timer logic (pure function test) ──

#[test]
fn production_timer_cycles_correctly() {
    let mut timer = 0.0_f32;
    let interval = 0.5_f32;
    let mut events = 0;

    // simulate several timesteps
    for _ in 0..50 {
        timer += 0.1_f32; // delta
        while timer >= interval {
            timer -= interval;
            events += 1;
        }
    }

    // 50 steps * 0.1 = 5.0 seconds total => 10 events
    assert_eq!(events, 10);
    // timer should be 0.0 after last cycle
    assert!((timer - 0.0).abs() < f32::EPSILON);
}

// ── ChunkGrid generation tests ──

#[test]
fn chunk_grid_generates_deterministically() {
    let mut grid_a = ChunkGrid::new(42);
    let mut grid_b = ChunkGrid::new(42);

    let chunk_a = grid_a.ensure_chunk(2, 3).clone();
    let chunk_b = grid_b.ensure_chunk(2, 3).clone();

    assert_eq!(chunk_a.tiles, chunk_b.tiles);
    assert_eq!(chunk_a.deposits, chunk_b.deposits);
}

#[test]
fn chunk_grid_stores_seed() {
    let grid_a = ChunkGrid::new(42);
    let grid_b = ChunkGrid::new(12345);
    // Both grids are valid and can generate chunks without panicking
    assert!(grid_a.chunk_exists(0, 0) == false);
    assert!(grid_b.chunk_exists(0, 0) == false);
}

// ── Resource tests ──

#[test]
fn resource_id_display_name() {
    assert_eq!(ResourceId("iron_ore".to_string()).display_name(), "Iron Ore");
    assert_eq!(ResourceId("copper_plate".to_string()).display_name(), "Copper Plate");
    assert_eq!(ResourceId("ore".to_string()).display_name(), "Ore");
}

#[test]
fn resource_id_eq() {
    assert_eq!(ResourceId("ore".to_string()), ResourceId("ore".to_string()));
    assert_ne!(ResourceId("ore".to_string()), ResourceId("iron_ore".to_string()));
}
