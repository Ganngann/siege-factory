use siege_factory::economy::building::BuildingRegistry;
use siege_factory::economy::unit_config::UnitConfig;
use siege_factory::economy::resource::{ResourceId, Inventory};
use siege_factory::map::config::MapConfig;
use siege_factory::map::components::TilePosition;
use siege_factory::economy::components::Produces;

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

// ── Production ECS test ──

#[test]
fn production_tick_emits_events() {
    use bevy::prelude::*;
    use bevy::app::MinimalPlugins;
    use siege_factory::economy::production::production_tick;
    use siege_factory::events::SpawnBeltItemEvent;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(MapConfig::load());
    app.add_event::<SpawnBeltItemEvent>();
    app.add_systems(Update, production_tick);

    let producer = app.world_mut().spawn((
        Produces { resource: ResourceId::Ore, interval: 0.5, timer: 0.0 },
        TilePosition { x: 5, y: 5 },
    )).id();

    // tick once — timer < interval
    app.update();
    {
        let prod = app.world().get::<Produces>(producer).unwrap();
        assert!(prod.timer < prod.interval,
            "Timer should be less than interval after one tick");
    }

    // tick 60 times at ~1/60s each → enough to cycle
    for _ in 0..120 {
        app.update();
    }

    let events = app.world().resource::<Events<SpawnBeltItemEvent>>();
    let mut reader = events.get_reader();
    let count = reader.read(&events).count();
    assert!(count > 0,
        "Expected at least 1 production event after 120 ticks, got {}", count);
}

// ── Combat ECS test ──

#[test]
fn turret_shoots_within_range() {
    use bevy::prelude::*;
    use bevy::app::MinimalPlugins;
    use siege_factory::enemy::combat::turret_shoot;
    use siege_factory::enemy::components::{Enemy, Health};
    use siege_factory::economy::components::TurretCombat;
    use siege_factory::combat::Projectile;
    use siege_factory::events::DespawnEnemy;
    use siege_factory::rendering::ShapeCache;
    use bevy::asset::AssetPlugin;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_event::<DespawnEnemy>();
    app.init_resource::<ShapeCache>();
    app.add_systems(Update, turret_shoot);

    // Place turret at origin
    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat { damage: 10, range_sq: 400.0, fire_interval: 0.1, timer: 5.0 },
    ));

    // Place enemy at (5, 0) — within range (400 = 20², 5² = 25 < 400)
    let enemy = app.world_mut().spawn((
        Enemy,
        Health { current: 50, max: 50 },
        Transform::from_xyz(5.0, 0.0, 0.0),
    )).id();

    // Place enemy at (50, 0) — out of range (50² = 2500 > 400)
    app.world_mut().spawn((
        Enemy,
        Health { current: 50, max: 50 },
        Transform::from_xyz(50.0, 0.0, 0.0),
    ));

    app.update();

    let projectiles = app.world().query::<&Projectile>().iter(&app.world())
        .collect::<Vec<_>>();
    assert_eq!(projectiles.len(), 1,
        "Expected 1 projectile (target in range), got {}", projectiles.len());
    assert_eq!(projectiles[0].target, enemy,
        "Projectile should target the closest enemy");
    assert_eq!(projectiles[0].damage, 10,
        "Projectile damage should match TurretCombat.damage");
}

#[test]
fn turret_does_not_shoot_without_enemies() {
    use bevy::prelude::*;
    use bevy::app::MinimalPlugins;
    use siege_factory::enemy::combat::turret_shoot;
    use siege_factory::economy::components::TurretCombat;
    use siege_factory::combat::Projectile;
    use siege_factory::events::DespawnEnemy;
    use siege_factory::rendering::ShapeCache;
    use bevy::asset::AssetPlugin;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_event::<DespawnEnemy>();
    app.init_resource::<ShapeCache>();
    app.add_systems(Update, turret_shoot);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        TurretCombat { damage: 5, range_sq: 400.0, fire_interval: 0.1, timer: 5.0 },
    ));

    app.update();

    let projectiles = app.world().query::<&Projectile>().iter(&app.world())
        .collect::<Vec<_>>();
    assert!(projectiles.is_empty(),
        "Turret should not shoot when no enemies are present");
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
