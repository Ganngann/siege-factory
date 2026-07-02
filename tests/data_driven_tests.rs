use siege_factory::economy::building::BuildingRegistry;
use siege_factory::economy::unit_config::UnitConfig;
use siege_factory::economy::resource::{ResourceId, Inventory};
use siege_factory::map::config::MapConfig;

// ── Registry tests (pure data, no ECS) ──

#[test]
fn all_buildings_have_visual() {
    let registry = BuildingRegistry::load();
    for def in &registry.buildings {
        if def.id == "hq" {
            continue;
        }
        assert!(!def.visual.is_empty(),
            "Building '{}' lacks visual field", def.id);
    }
}

#[test]
fn all_buildings_have_valid_cost() {
    let registry = BuildingRegistry::load();
    for def in &registry.buildings {
        for cost in &def.cost {
            assert!(cost.amount > 0,
                "Building '{}' has zero-cost resource {:?}", def.id, cost.resource);
        }
    }
}

#[test]
fn combat_buildings_have_combat_stats() {
    let registry = BuildingRegistry::load();
    for def in &registry.buildings {
        if def.combat.is_some() {
            let combat = def.combat.as_ref().unwrap();
            assert!(combat.damage > 0, "{} combat damage is 0", def.id);
            assert!(combat.range > 0.0, "{} combat range is 0", def.id);
            assert!(combat.fire_rate_sec > 0.0, "{} combat fire_rate is 0", def.id);
        }
    }
}

#[test]
fn deposit_buildings_require_deposit() {
    let registry = BuildingRegistry::load();
    let deposit_buildings: Vec<_> = registry.buildings.iter()
        .filter(|b| b.requires_deposit).collect();
    assert!(!deposit_buildings.is_empty(), "No buildings require a deposit");
    for b in &deposit_buildings {
        assert!(b.production.is_some(),
            "Building '{}' requires deposit but has no production", b.id);
    }
}

#[test]
fn production_buildings_have_production_def() {
    let registry = BuildingRegistry::load();
    for def in &registry.buildings {
        if let Some(prod) = &def.production {
            assert!(prod.interval_sec > 0.0,
                "Building '{}' production interval is 0", def.id);
        }
    }
}

#[test]
fn belt_buildings_have_belt_properties() {
    let registry = BuildingRegistry::load();
    for def in &registry.buildings {
        if let Some(belt) = &def.belt {
            assert!(belt.slots > 0, "Building '{}' belt has 0 slots", def.id);
            assert!(belt.speed > 0.0, "Building '{}' belt speed is 0", def.id);
        }
    }
}

#[test]
fn all_units_have_visual() {
    let cfg = UnitConfig::load();
    for (id, def) in &cfg.units {
        assert!(!def.visual.is_empty(),
            "Unit '{}' lacks visual field", id);
    }
}

#[test]
fn all_units_have_valid_cost() {
    let cfg = UnitConfig::load();
    for (id, def) in &cfg.units {
        for cost in &def.cost {
            assert!(cost.amount > 0,
                "Unit '{}' has zero-cost resource {:?}", id, cost.resource);
        }
    }
}

#[test]
fn combat_units_have_damage() {
    let cfg = UnitConfig::load();
    for (id, def) in &cfg.units {
        if def.kind == "combat" {
            assert!(def.damage > 0, "Combat unit '{}' has 0 damage", id);
            assert!(def.range_tiles > 0.0, "Combat unit '{}' has 0 range", id);
        }
    }
}

#[test]
fn harvester_units_have_harvest_stats() {
    let cfg = UnitConfig::load();
    for (id, def) in &cfg.units {
        if def.kind == "harvester" {
            assert!(def.speed > 0.0, "Harvester '{}' has 0 speed", id);
            assert!(def.mine_interval_sec > 0.0,
                "Harvester '{}' has 0 mine interval", id);
        }
    }
}

#[test]
fn building_registry_contains_all_toml_entries() {
    let registry = BuildingRegistry::load();
    let ids: Vec<&str> = registry.buildings.iter().map(|b| b.id.as_str()).collect();
    assert!(ids.contains(&"hq"), "Missing HQ");
    assert!(ids.contains(&"miner"), "Missing miner");
    assert!(ids.contains(&"belt"), "Missing belt");
    assert!(ids.contains(&"turret"), "Missing turret");
    assert!(ids.len() >= 5, "Expected at least 5 buildings, got {}", ids.len());
}

#[test]
fn unit_config_contains_all_toml_entries() {
    let cfg = UnitConfig::load();
    assert!(cfg.units.contains_key("soldier"), "Missing soldier");
    assert!(cfg.units.contains_key("worker"), "Missing worker");
    assert!(cfg.units.len() >= 2, "Expected at least 2 units, got {}", cfg.units.len());
}

// ── Economy tests ──

#[test]
fn inventory_operations() {
    let mut inv = Inventory::new();
    assert_eq!(inv.get(ResourceId::Ore), 0);

    inv.add(ResourceId::Ore, 10);
    assert_eq!(inv.get(ResourceId::Ore), 10);

    inv.add(ResourceId::Ore, 5);
    assert_eq!(inv.get(ResourceId::Ore), 15);

    assert!(inv.remove(ResourceId::Ore, 7));
    assert_eq!(inv.get(ResourceId::Ore), 8);

    assert!(!inv.remove(ResourceId::Ore, 20));
    assert_eq!(inv.get(ResourceId::Ore), 8);
}

#[test]
fn inventory_saturating_add() {
    let mut inv = Inventory::new();
    inv.add(ResourceId::Ore, u32::MAX);
    inv.add(ResourceId::Ore, 100);
    assert_eq!(inv.get(ResourceId::Ore), u32::MAX);
}

#[test]
fn inventory_separate_resources() {
    let mut inv = Inventory::new();
    inv.add(ResourceId::Ore, 10);
    inv.add(ResourceId::Ammo, 5);
    inv.add(ResourceId::Energy, 3);
    assert_eq!(inv.get(ResourceId::Ore), 10);
    assert_eq!(inv.get(ResourceId::Ammo), 5);
    assert_eq!(inv.get(ResourceId::Energy), 3);
    inv.remove(ResourceId::Ore, 10);
    assert_eq!(inv.get(ResourceId::Ore), 0);
    assert_eq!(inv.get(ResourceId::Ammo), 5);
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

    assert!(events > 0, "Timer should have cycled at least once");
    assert!(timer < interval, "Timer should end below interval");
}

// ── Combat targeting logic (pure function tests) ──

#[test]
fn turret_find_closest_enemy_in_range() {
    use bevy::prelude::*;
    use siege_factory::enemy::combat::find_closest_enemy;

    let turret_pos = Vec3::ZERO;
    let enemies = vec![
        (Entity::from_bits(1), Vec3::new(5.0, 0.0, 0.0)),   // in range (25 < 400)
        (Entity::from_bits(2), Vec3::new(50.0, 0.0, 0.0)),  // out of range
    ];

    let result = find_closest_enemy(turret_pos, &enemies, 400.0);
    assert_eq!(result, Some(Entity::from_bits(1)),
        "Should target the closest enemy in range");
}

#[test]
fn turret_no_enemy_in_range_returns_none() {
    use bevy::prelude::*;
    use siege_factory::enemy::combat::find_closest_enemy;

    let turret_pos = Vec3::ZERO;
    let enemies = vec![
        (Entity::from_bits(1), Vec3::new(50.0, 0.0, 0.0)),   // out of range
    ];

    let result = find_closest_enemy(turret_pos, &enemies, 400.0);
    assert_eq!(result, None,
        "Should return None when no enemy is in range");
}

#[test]
fn turret_empty_enemies_returns_none() {
    use bevy::prelude::*;
    use siege_factory::enemy::combat::find_closest_enemy;

    let turret_pos = Vec3::ZERO;
    let enemies = vec![];

    let result = find_closest_enemy(turret_pos, &enemies, 400.0);
    assert_eq!(result, None,
        "Should return None when there are no enemies");
}

// ── BuildingDef data sanity ──

#[test]
fn map_config_is_valid() {
    let cfg = MapConfig::load();
    assert!(cfg.width > 0, "Map width must be > 0");
    assert!(cfg.height > 0, "Map height must be > 0");
    assert!(cfg.tile_size > 0.0, "Tile size must be > 0");
    assert!(cfg.hq_start_ore > 0, "HQ must start with ore");
    assert!(cfg.hq_hp > 0, "HQ must have HP");
    assert!(!cfg.deposit_positions.is_empty(), "Must have at least 1 deposit");
    assert!(cfg.deposit_max_amount > 0, "Deposits must have amount");
}
