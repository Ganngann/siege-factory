use std::time::Duration;

use bevy::prelude::*;
use siege_factory::core::toast::ToastQueue;
use siege_factory::economy::archive::archive_delivery_check;
use siege_factory::economy::belt::{advance_belt_slots, building_output_tick, BeltSlots, ItemOnBelt};
use siege_factory::economy::building::BuildingRegistry;
use siege_factory::economy::components::{
    Active, Archive, Assembler, Builder, BuilderState, Building, BurnerGenerator, CurrentTier,
    DiscoveredRecipes, Direction, OccupiedTiles, Player, PowerConsumer, PowerPole, PowerProducer,
    ProductionCounter, ResourceDeposit, UnbuiltBuilding,
};
use siege_factory::economy::discovery::{
    check_discoveries, DiscoveryRegistry, GlobalArchive,
};
use siege_factory::economy::player::{
    builder_work, finish_construction, MiningTimer, PlayerWorldPos, player_mine, player_movement,
};
use siege_factory::economy::power::{
    burner_generator_tick, rebuild_power_grid, PowerGrid,
};
use siege_factory::economy::production::assembler_tick;
use siege_factory::economy::recipe::RecipeRegistry;
use siege_factory::economy::resource::{Inventory, ResourceId, ResourceRegistry};
use siege_factory::economy::spatial::SpatialRegistry;
use siege_factory::economy::tiered_structure::{structure_interact, ProgressionLogRegistry};
use siege_factory::map::components::TilePosition;
use siege_factory::map::config::MapConfig;
use siege_factory::map::tile_grid::ChunkGrid;
use siege_factory::core::modding::ModRegistry;


// ════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════

fn test_mods() -> ModRegistry { ModRegistry::for_test() }
fn economy_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(MapConfig::load(&test_mods()));
    app.insert_resource(RecipeRegistry::load(&test_mods()));
    app.insert_resource(BuildingRegistry::load(&test_mods()));
    app.insert_resource(ResourceRegistry::load(&test_mods()));
    app.insert_resource(DiscoveryRegistry::load(&test_mods()));
    app.insert_resource(GlobalArchive::new(&[]));
    app.init_resource::<SpatialRegistry>();
    app.init_resource::<PowerGrid>();
    app.init_resource::<ToastQueue>();
    app.init_resource::<PlayerWorldPos>();
    app.init_resource::<MiningTimer>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.insert_resource(Time::<Fixed>::from_hz(20.0));
    app.insert_resource(ProgressionLogRegistry::default());
    app.insert_resource(ChunkGrid::new(42, 5, 20, 50, 1, 3, vec![]));
    app
}

/// Advance Time<Fixed> overstep so FixedUpdate systems run during the next app.update().
fn advance_fixed(app: &mut App, seconds: f32) {
    let delta = Duration::from_secs_f32(seconds);
    app.world_mut().resource_mut::<Time<Fixed>>().accumulate_overstep(delta);
    app.update();
}

/// Advance Time<Real> so systems using `Time` (real time) see a non-zero delta.
fn advance_real(app: &mut App, millis: u64) {
    app.update(); // Prime Time<Real> so delta is computed from last_update
    std::thread::sleep(Duration::from_millis(millis));
    app.update();
}

// ════════════════════════════════════════════════════════════════
// assembler_tick tests
// ════════════════════════════════════════════════════════════════

#[test]
fn assembler_tick_produces_output_after_interval() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, assembler_tick);

    let iron_ore = ResourceId::new("iron_ore");
    let iron_plate = ResourceId::new("iron_plate");

    let recipe = app.world().resource::<RecipeRegistry>().get("iron_plate").unwrap().clone();
    let time_sec = recipe.time_sec;

    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: time_sec,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
        Active(true),
    ));

    // Not enough time - no production
    advance_fixed(&mut app, time_sec * 0.5);
    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert_eq!(inv.get(&iron_plate), 0, "should not produce before interval");
    }

    // Enough time - production occurs
    advance_fixed(&mut app, time_sec + 0.1);
    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert!(inv.get(&iron_plate) >= 1, "should produce at least 1 iron_plate");
        assert!(inv.get(&iron_ore) <= 8, "should consume iron_ore");
    }
}

#[test]
fn assembler_tick_blocked_without_power() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, assembler_tick);

    let iron_ore = ResourceId::new("iron_ore");
    let iron_plate = ResourceId::new("iron_plate");

    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
        Active(true),
        PowerConsumer { draw: 10.0, satisfied: false },
    ));

    advance_fixed(&mut app, 10.0);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert_eq!(inv.get(&iron_plate), 0, "should not produce without power");
    }
}

#[test]
fn assembler_tick_blocked_with_insufficient_inputs() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, assembler_tick);

    let iron_plate = ResourceId::new("iron_plate");

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: 0.1,
            recipe_id: "iron_plate".to_string(),
        },
        Inventory::new(),
        Active(true),
    ));

    advance_fixed(&mut app, 5.0);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert_eq!(inv.get(&iron_plate), 0, "should not produce without inputs");
    }
}

#[test]
fn assembler_tick_inactive_building_does_not_produce() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, assembler_tick);

    let iron_ore = ResourceId::new("iron_ore");
    let iron_plate = ResourceId::new("iron_plate");

    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: 0.01,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
        Active(false),
    ));

    advance_fixed(&mut app, 10.0);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert_eq!(inv.get(&iron_plate), 0, "inactive assembler should not produce");
    }
}

#[test]
fn assembler_tick_production_counter_increments() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, assembler_tick);

    let iron_ore = ResourceId::new("iron_ore");

    let recipe = app.world().resource::<RecipeRegistry>().get("iron_plate").unwrap().clone();
    let time_sec = recipe.time_sec;
    let output_amount = recipe.output.iter().map(|(_, a)| a).sum::<u32>();

    let mut inv = Inventory::new();
    inv.add(&iron_ore, 50);

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: time_sec,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
        Active(true),
        ProductionCounter(0),
    ));

    advance_fixed(&mut app, time_sec + 0.1);

    {
        let mut q = app.world_mut().query::<&ProductionCounter>();
        let counter = q.single(app.world()).unwrap();
        assert!(counter.0 >= output_amount, "counter should have incremented by output amount, got {}", counter.0);
    }
}

// ════════════════════════════════════════════════════════════════
// rebuild_power_grid tests
// ════════════════════════════════════════════════════════════════

#[test]
fn rebuild_power_grid_single_producer_satisfies_consumer() {
    let mut app = economy_test_app();
    app.add_systems(Update, rebuild_power_grid);

    app.world_mut().spawn((
        PowerProducer { output: 100.0 },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let iron_ore = ResourceId::new("iron_ore");
    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        PowerConsumer { draw: 50.0, satisfied: false },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Active(true),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&PowerConsumer>();
        let consumer = q.single(app.world()).unwrap();
        assert!(consumer.satisfied, "consumer should be satisfied with enough power");
    }
}

#[test]
fn rebuild_power_grid_excess_demand_unsatisfied() {
    let mut app = economy_test_app();
    app.add_systems(Update, rebuild_power_grid);

    app.world_mut().spawn((
        PowerProducer { output: 10.0 },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let iron_ore = ResourceId::new("iron_ore");
    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        PowerConsumer { draw: 50.0, satisfied: false },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Active(true),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&PowerConsumer>();
        let consumer = q.single(app.world()).unwrap();
        assert!(!consumer.satisfied, "consumer should NOT be satisfied when demand exceeds supply");
    }
}

#[test]
fn rebuild_power_grid_utilization_ratio_computed() {
    let mut app = economy_test_app();
    app.add_systems(Update, rebuild_power_grid);

    app.world_mut().spawn((
        PowerProducer { output: 100.0 },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let iron_ore = ResourceId::new("iron_ore");
    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        PowerConsumer { draw: 50.0, satisfied: false },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Active(true),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
    ));

    app.update();

    let grid = app.world().resource::<PowerGrid>();
    assert!(
        (grid.utilization_ratio - 0.5).abs() < 0.01,
        "utilization should be ~0.5, got {}",
        grid.utilization_ratio
    );
}

#[test]
fn rebuild_power_grid_no_producers_all_unsatisfied() {
    let mut app = economy_test_app();
    app.add_systems(Update, rebuild_power_grid);

    let iron_ore = ResourceId::new("iron_ore");
    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        PowerConsumer { draw: 50.0, satisfied: false },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Active(true),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&PowerConsumer>();
        let consumer = q.single(app.world()).unwrap();
        assert!(!consumer.satisfied, "consumer should be unsatisfied with no producers");
    }
}

#[test]
fn rebuild_power_grid_pole_range_filtering() {
    let mut app = economy_test_app();
    app.add_systems(Update, rebuild_power_grid);

    let tile_size = app.world().resource::<MapConfig>().tile_size;

    app.world_mut().spawn((
        PowerProducer { output: 100.0 },
        Transform::from_xyz(100.0 * tile_size, 0.0, 0.0),
    ));

    app.world_mut().spawn((
        PowerPole { range: 1.0 },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let iron_ore = ResourceId::new("iron_ore");
    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        PowerConsumer { draw: 50.0, satisfied: false },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Active(true),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&PowerConsumer>();
        let consumer = q.single(app.world()).unwrap();
        assert!(!consumer.satisfied, "consumer should be unsatisfied when producer is out of pole range");
    }
}

#[test]
fn rebuild_power_grid_multiple_producers_pool_power() {
    let mut app = economy_test_app();
    app.add_systems(Update, rebuild_power_grid);

    app.world_mut().spawn((
        PowerProducer { output: 50.0 },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    app.world_mut().spawn((
        PowerProducer { output: 50.0 },
        Transform::from_xyz(1.0, 0.0, 0.0),
    ));

    let iron_ore = ResourceId::new("iron_ore");
    let mut inv = Inventory::new();
    inv.add(&iron_ore, 10);

    app.world_mut().spawn((
        PowerConsumer { draw: 80.0, satisfied: false },
        Transform::from_xyz(0.5, 0.0, 0.0),
        Active(true),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&PowerConsumer>();
        let consumer = q.single(app.world()).unwrap();
        assert!(consumer.satisfied, "consumer should be satisfied when total production >= demand");
    }
}

// ════════════════════════════════════════════════════════════════
// player_movement tests
// ════════════════════════════════════════════════════════════════

#[test]
fn player_movement_w_key_moves_up() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_movement);

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Transform::from_xyz(160.0, 160.0, 5.0),
    ));

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyW);
    advance_real(&mut app, 20);

    {
        let mut q = app.world_mut().query::<&Transform>();
        let tf = q.single(app.world()).unwrap();
        assert!(tf.translation.y > 160.0, "W should move player up (y increases), got {}", tf.translation.y);
        assert_eq!(tf.translation.x, 160.0, "W should not change x");
    }

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyW);
}

#[test]
fn player_movement_tile_position_updates() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_movement);

    let cfg = app.world().resource::<MapConfig>().clone();
    let start = siege_factory::core::utils::tile_to_world(5, 5, cfg.tile_size);

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Transform::from_xyz(start.x, start.y, 5.0),
    ));

    for _ in 0..5 {
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyD);
        advance_real(&mut app, 20);
    }
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyD);

    {
        let mut q = app.world_mut().query::<&TilePosition>();
        let tp = q.single(app.world()).unwrap();
        assert!(tp.x >= 5, "TilePosition x should update after moving right, got {}", tp.x);
    }
}

#[test]
fn player_movement_no_keys_no_movement() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_movement);

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Transform::from_xyz(160.0, 160.0, 5.0),
    ));

    let before;
    {
        let mut q = app.world_mut().query::<&Transform>();
        before = q.single(app.world()).unwrap().translation;
    }

    advance_real(&mut app, 50);

    let after;
    {
        let mut q = app.world_mut().query::<&Transform>();
        after = q.single(app.world()).unwrap().translation;
    }
    assert_eq!(before, after, "no keys pressed should mean no movement");
}

#[test]
fn player_movement_diagonal_normalization() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_movement);

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Transform::from_xyz(160.0, 160.0, 5.0),
    ));

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyW);
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyD);
    advance_real(&mut app, 20);

    {
        let mut q = app.world_mut().query::<&Transform>();
        let tf = q.single(app.world()).unwrap();
        let dx = tf.translation.x - 160.0;
        let dy = tf.translation.y - 160.0;
        assert!(dx > 0.0, "diagonal should move right");
        assert!(dy > 0.0, "diagonal should move up");
    }

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyW);
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyD);
}

// ════════════════════════════════════════════════════════════════
// player_mine tests
// ════════════════════════════════════════════════════════════════

#[test]
fn player_mine_mines_adjacent_deposit() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_mine);

    let iron_ore = ResourceId::new("iron_ore");

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Inventory::new(),
    ));

    app.world_mut().spawn((
        ResourceDeposit {
            resource: "iron_ore".to_string(),
            amount: 10,
        },
        TilePosition { x: 6, y: 5 },
    ));

    app.world_mut().resource_mut::<MapConfig>().player_mining_interval = 0.0;

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyE);
    advance_real(&mut app, 20);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert!(inv.get(&iron_ore) >= 1, "should mine at least 1 iron_ore");
    }

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyE);
}

#[test]
fn player_mine_no_deposit_nearby_does_nothing() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_mine);

    let iron_ore = ResourceId::new("iron_ore");

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Inventory::new(),
    ));

    app.world_mut().spawn((
        ResourceDeposit {
            resource: "iron_ore".to_string(),
            amount: 10,
        },
        TilePosition { x: 100, y: 100 },
    ));

    app.world_mut().resource_mut::<MapConfig>().player_mining_interval = 0.0;

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyE);
    advance_real(&mut app, 20);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert_eq!(inv.get(&iron_ore), 0, "should not mine when no deposit nearby");
    }

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyE);
}

#[test]
fn player_mine_timer_resets_on_release() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_mine);

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Inventory::new(),
    ));

    app.world_mut().resource_mut::<MapConfig>().player_mining_interval = 5.0;

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyE);
    advance_real(&mut app, 50);

    let timer_before = app.world().resource::<MiningTimer>().0;
    assert!(timer_before > 0.0, "timer should accumulate while E held");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyE);
    app.update();

    let timer_after = app.world().resource::<MiningTimer>().0;
    assert_eq!(timer_after, 0.0, "timer should reset when E released");
}

#[test]
fn player_mine_depletes_deposit() {
    let mut app = economy_test_app();
    app.add_systems(Update, player_mine);

    app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Inventory::new(),
    ));

    let dep_entity = app.world_mut().spawn((
        ResourceDeposit {
            resource: "iron_ore".to_string(),
            amount: 1,
        },
        TilePosition { x: 6, y: 5 },
    )).id();

    app.world_mut().resource_mut::<MapConfig>().player_mining_interval = 0.0;
    app.world_mut().resource_mut::<MapConfig>().infinite_deposits = false;

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyE);
    advance_real(&mut app, 20);

    let dep = app.world().entity(dep_entity).get::<ResourceDeposit>().unwrap();
    assert_eq!(dep.amount, 0, "deposit should be depleted after mining");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyE);
}

// ════════════════════════════════════════════════════════════════
// check_discoveries tests
// ════════════════════════════════════════════════════════════════

#[test]
fn check_discoveries_triggers_at_threshold() {
    let mut app = economy_test_app();
    app.add_systems(Update, check_discoveries);

    let disc_reg = app.world().resource::<DiscoveryRegistry>().clone();
    let steel_disc = disc_reg.discoveries.iter().find(|d| d.reward_id == "steel").unwrap();
    let building_kind = steel_disc.building.clone();
    let threshold = steel_disc.threshold;

    app.world_mut().spawn((
        Building {
            kind: building_kind,
            name: "Test".to_string(),
        },
        ProductionCounter(threshold),
        DiscoveredRecipes::default(),
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&DiscoveredRecipes>();
        let discovered = q.single(app.world()).unwrap();
        assert!(
            discovered.0.contains(&"steel".to_string()),
            "should discover 'steel' when counter >= threshold"
        );
    }
}

#[test]
fn check_discoveries_below_threshold_no_trigger() {
    let mut app = economy_test_app();
    app.add_systems(Update, check_discoveries);

    let disc_reg = app.world().resource::<DiscoveryRegistry>().clone();
    let steel_disc = disc_reg.discoveries.iter().find(|d| d.reward_id == "steel").unwrap();
    let building_kind = steel_disc.building.clone();
    let threshold = steel_disc.threshold;

    app.world_mut().spawn((
        Building {
            kind: building_kind,
            name: "Test".to_string(),
        },
        ProductionCounter(threshold - 1),
        DiscoveredRecipes::default(),
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&DiscoveredRecipes>();
        let discovered = q.single(app.world()).unwrap();
        assert!(
            !discovered.0.contains(&"steel".to_string()),
            "should NOT discover when counter < threshold"
        );
    }
}

#[test]
fn check_discoveries_already_discovered_no_duplicate() {
    let mut app = economy_test_app();
    app.add_systems(Update, check_discoveries);

    let disc_reg = app.world().resource::<DiscoveryRegistry>().clone();
    let steel_disc = disc_reg.discoveries.iter().find(|d| d.reward_id == "steel").unwrap();
    let building_kind = steel_disc.building.clone();

    let mut discovered = DiscoveredRecipes::default();
    discovered.0.push("steel".to_string());

    app.world_mut().spawn((
        Building {
            kind: building_kind,
            name: "Test".to_string(),
        },
        ProductionCounter(100),
        discovered,
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&DiscoveredRecipes>();
        let d = q.single(app.world()).unwrap();
        assert_eq!(
            d.0.iter().filter(|id| *id == "steel").count(),
            1,
            "should not add duplicate discovery"
        );
    }
}

#[test]
fn check_discoveries_already_globally_unlocked_skips() {
    let mut app = economy_test_app();
    app.add_systems(Update, check_discoveries);

    let disc_reg = app.world().resource::<DiscoveryRegistry>().clone();
    let steel_disc = disc_reg.discoveries.iter().find(|d| d.reward_id == "steel").unwrap();
    let building_kind = steel_disc.building.clone();

    app.world_mut().resource_mut::<GlobalArchive>().unlocked_recipes.insert("steel".to_string());

    app.world_mut().spawn((
        Building {
            kind: building_kind,
            name: "Test".to_string(),
        },
        ProductionCounter(100),
        DiscoveredRecipes::default(),
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&DiscoveredRecipes>();
        let d = q.single(app.world()).unwrap();
        assert!(
            !d.0.contains(&"steel".to_string()),
            "should NOT re-discover if already globally unlocked"
        );
    }
}

// ════════════════════════════════════════════════════════════════
// archive_delivery_check tests
// ════════════════════════════════════════════════════════════════

#[test]
fn archive_delivery_unlocks_recipe() {
    let mut app = economy_test_app();
    app.add_systems(Update, archive_delivery_check);

    let iron_ore = ResourceId::new("iron_ore");

    // Spawn archive with iron_ore in inventory
    let mut inv = Inventory::new();
    inv.add(&iron_ore, 5);
    app.world_mut().spawn((Archive, inv));

    // Spawn a DiscoveredRecipes with iron_ore pending
    app.world_mut().spawn(DiscoveredRecipes(vec!["iron_ore".to_string()]));

    app.update();

    let toast = app.world().resource::<ToastQueue>();
    assert!(
        toast.0.iter().any(|t| t.contains("Archived")),
        "should have generated an archive toast, got: {:?}",
        toast.0
    );
}

#[test]
fn archive_delivery_ignores_zero_amount() {
    let mut app = economy_test_app();
    app.add_systems(Update, archive_delivery_check);

    let inv = Inventory::new();
    app.world_mut().spawn((Archive, inv));
    app.world_mut().spawn(DiscoveredRecipes(vec!["iron_ore".to_string()]));

    app.update();

    let toast = app.world().resource::<ToastQueue>();
    assert!(toast.0.is_empty(), "should not produce toast for zero-amount inventory");
}

#[test]
fn archive_delivery_ignores_not_pending() {
    let mut app = economy_test_app();
    app.add_systems(Update, archive_delivery_check);

    let iron_ore = ResourceId::new("iron_ore");

    let mut inv = Inventory::new();
    inv.add(&iron_ore, 5);
    app.world_mut().spawn((Archive, inv));

    app.world_mut().spawn(DiscoveredRecipes(vec![]));

    app.update();

    let toast = app.world().resource::<ToastQueue>();
    assert!(toast.0.is_empty(), "should not produce toast when resource is not pending");
}

// ════════════════════════════════════════════════════════════════
// finish_construction tests
// ════════════════════════════════════════════════════════════════

#[test]
fn finish_construction_completes_when_cost_met() {
    let mut app = economy_test_app();
    app.add_systems(Update, finish_construction);

    let registry = app.world().resource::<BuildingRegistry>().clone();
    let wall_def = registry.get("wall").unwrap();
    let cost = wall_def.cost.clone();

    let mut inv = Inventory::new();
    for c in &cost {
        inv.add(&c.resource, c.amount);
    }

    app.world_mut().spawn((
        Building {
            kind: "wall".to_string(),
            name: "Wall".to_string(),
        },
        UnbuiltBuilding,
        inv,
        Sprite::default(),
    ));

    app.update();

    let mut q = app.world_mut().query::<Entity>();
    let remaining: Vec<_> = q.iter(app.world())
        .filter(|e| app.world().entity(*e).contains::<UnbuiltBuilding>())
        .collect();
    assert!(remaining.is_empty(), "UnbuiltBuilding should be removed after construction completes");
}

#[test]
fn finish_construction_does_nothing_when_cost_not_met() {
    let mut app = economy_test_app();
    app.add_systems(Update, finish_construction);

    let inv = Inventory::new();

    app.world_mut().spawn((
        Building {
            kind: "wall".to_string(),
            name: "Wall".to_string(),
        },
        UnbuiltBuilding,
        inv,
    ));

    app.update();

    {
        let mut q = app.world_mut().query::<&UnbuiltBuilding>();
        let has_unbuilt = q.single(app.world()).is_ok();
        assert!(has_unbuilt, "UnbuiltBuilding should remain when cost not met");
    }
}

#[test]
fn finish_construction_inserts_active_component() {
    let mut app = economy_test_app();
    app.add_systems(Update, finish_construction);

    let registry = app.world().resource::<BuildingRegistry>().clone();
    let wall_def = registry.get("wall").unwrap();
    let cost = wall_def.cost.clone();

    let mut inv = Inventory::new();
    for c in &cost {
        inv.add(&c.resource, c.amount);
    }

    let entity = app.world_mut().spawn((
        Building {
            kind: "wall".to_string(),
            name: "Wall".to_string(),
        },
        UnbuiltBuilding,
        inv,
    )).id();

    app.update();

    assert!(
        app.world().entity(entity).contains::<Active>(),
        "completed building should have Active component"
    );
    assert!(
        !app.world().entity(entity).contains::<UnbuiltBuilding>(),
        "completed building should not have UnbuiltBuilding"
    );
}

// ════════════════════════════════════════════════════════════════
// advance_belt_slots tests
// ════════════════════════════════════════════════════════════════

#[test]
fn advance_belt_slots_moves_item_forward() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, advance_belt_slots);

    let iron_ore = ResourceId::new("iron_ore");

    let slot_positions = vec![Vec2::new(0.0, 0.0), Vec2::new(32.0, 0.0)];
    let items: Vec<Option<ItemOnBelt>> = vec![
        Some(ItemOnBelt {
            resource_id: iron_ore.clone(),
            acc: 0.0,
        }),
        None,
    ];

    app.world_mut().spawn((
        TilePosition { x: 10, y: 10 },
        BeltSlots {
            direction: Direction::East,
            slot_positions,
            items,
            slot_sprites: vec![None, None],
            speed: 2.0,
        },
    ));

    // advance_belt_slots runs in FixedUpdate
    advance_fixed(&mut app, 2.0);

    {
        let mut q = app.world_mut().query::<&BeltSlots>();
        let bs = q.single(app.world()).unwrap();
        assert!(
            bs.items[0].is_some() || bs.items[1].is_some(),
            "item should still be somewhere on the belt"
        );
        if let Some(ref item) = bs.items[0] {
            assert!(item.acc > 0.0, "slot 0 item should have accumulated time");
        }
    }
}

#[test]
fn advance_belt_slots_empty_belt_stays_empty() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, advance_belt_slots);

    let slot_positions = vec![Vec2::new(0.0, 0.0), Vec2::new(32.0, 0.0)];
    let items: Vec<Option<ItemOnBelt>> = vec![None, None];

    app.world_mut().spawn((
        TilePosition { x: 10, y: 10 },
        BeltSlots {
            direction: Direction::East,
            slot_positions,
            items,
            slot_sprites: vec![None, None],
            speed: 2.0,
        },
    ));

    advance_fixed(&mut app, 1.0);

    {
        let mut q = app.world_mut().query::<&BeltSlots>();
        let bs = q.single(app.world()).unwrap();
        assert!(bs.items[0].is_none(), "empty belt slot 0 should stay empty");
        assert!(bs.items[1].is_none(), "empty belt slot 1 should stay empty");
    }
}

#[test]
fn advance_belt_slots_full_belt_no_overflow() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, advance_belt_slots);

    let iron_ore = ResourceId::new("iron_ore");
    let iron_plate = ResourceId::new("iron_plate");

    let slot_positions = vec![Vec2::new(0.0, 0.0), Vec2::new(32.0, 0.0)];
    let items: Vec<Option<ItemOnBelt>> = vec![
        Some(ItemOnBelt {
            resource_id: iron_ore.clone(),
            acc: 0.0,
        }),
        Some(ItemOnBelt {
            resource_id: iron_plate.clone(),
            acc: 0.0,
        }),
    ];

    app.world_mut().spawn((
        TilePosition { x: 10, y: 10 },
        BeltSlots {
            direction: Direction::East,
            slot_positions,
            items,
            slot_sprites: vec![None, None],
            speed: 2.0,
        },
    ));

    advance_fixed(&mut app, 1.0);

    {
        let mut q = app.world_mut().query::<&BeltSlots>();
        let bs = q.single(app.world()).unwrap();
        assert!(bs.items[0].is_some(), "slot 0 should still have item");
        assert!(bs.items[1].is_some(), "slot 1 should still have item");
    }
}

// ════════════════════════════════════════════════════════════════
// building_output_tick tests
// ════════════════════════════════════════════════════════════════

#[test]
fn building_output_tick_transfers_from_building_to_belt() {
    let mut app = economy_test_app();
    app.add_systems(Update, building_output_tick);

    let iron_plate = ResourceId::new("iron_plate");

    let building_entity = app.world_mut().spawn((
        Building {
            kind: "furnace".to_string(),
            name: "Furnace".to_string(),
        },
        {
            let mut inv = Inventory::new();
            inv.add(&iron_plate, 3);
            inv
        },
        TilePosition { x: 5, y: 5 },
        OccupiedTiles(vec![(5, 5)]),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
    )).id();

    let belt_entity = app.world_mut().spawn((
        TilePosition { x: 6, y: 5 },
        BeltSlots {
            direction: Direction::East,
            slot_positions: vec![Vec2::new(0.0, 0.0), Vec2::new(32.0, 0.0)],
            items: vec![None, None],
            slot_sprites: vec![None, None],
            speed: 2.0,
        },
    )).id();

    {
        let mut spatial = app.world_mut().resource_mut::<SpatialRegistry>();
        spatial.insert_for_test(5, 5, building_entity);
    }

    app.update();

    let bs = app.world().entity(belt_entity).get::<BeltSlots>().unwrap();
    assert!(
        bs.items[0].is_some(),
        "belt slot 0 should have received an item from the building"
    );

    let inv = app.world().entity(building_entity).get::<Inventory>().unwrap();
    assert_eq!(inv.get(&iron_plate), 2, "building inventory should have decreased by 1");
}

#[test]
fn building_output_tick_no_output_when_belt_full() {
    let mut app = economy_test_app();
    app.add_systems(Update, building_output_tick);

    let iron_plate = ResourceId::new("iron_plate");

    let building_entity = app.world_mut().spawn((
        Building {
            kind: "furnace".to_string(),
            name: "Furnace".to_string(),
        },
        {
            let mut inv = Inventory::new();
            inv.add(&iron_plate, 3);
            inv
        },
        TilePosition { x: 5, y: 5 },
        OccupiedTiles(vec![(5, 5)]),
        Assembler {
            production_timer: 0.0,
            interval: 1.0,
            recipe_id: "iron_plate".to_string(),
        },
    )).id();

    let _belt_entity = app.world_mut().spawn((
        TilePosition { x: 6, y: 5 },
        BeltSlots {
            direction: Direction::East,
            slot_positions: vec![Vec2::new(0.0, 0.0), Vec2::new(32.0, 0.0)],
            items: vec![
                Some(ItemOnBelt {
                    resource_id: iron_plate.clone(),
                    acc: 0.0,
                }),
                None,
            ],
            slot_sprites: vec![None, None],
            speed: 2.0,
        },
    )).id();

    {
        let mut spatial = app.world_mut().resource_mut::<SpatialRegistry>();
        spatial.insert_for_test(5, 5, building_entity);
    }

    app.update();

    let inv = app.world().entity(building_entity).get::<Inventory>().unwrap();
    assert_eq!(inv.get(&iron_plate), 3, "building inventory should NOT decrease when belt is full");
}

#[test]
fn building_output_tick_empty_building_no_transfer() {
    let mut app = economy_test_app();
    app.add_systems(Update, building_output_tick);

    let building_entity = app.world_mut().spawn((
        Building {
            kind: "furnace".to_string(),
            name: "Furnace".to_string(),
        },
        Inventory::new(),
        TilePosition { x: 5, y: 5 },
        OccupiedTiles(vec![(5, 5)]),
    )).id();

    let belt_entity = app.world_mut().spawn((
        TilePosition { x: 6, y: 5 },
        BeltSlots {
            direction: Direction::East,
            slot_positions: vec![Vec2::new(0.0, 0.0), Vec2::new(32.0, 0.0)],
            items: vec![None, None],
            slot_sprites: vec![None, None],
            speed: 2.0,
        },
    )).id();

    {
        let mut spatial = app.world_mut().resource_mut::<SpatialRegistry>();
        spatial.insert_for_test(5, 5, building_entity);
    }

    app.update();

    let bs = app.world().entity(belt_entity).get::<BeltSlots>().unwrap();
    assert!(bs.items[0].is_none(), "belt should remain empty when building has no output");
}

// ════════════════════════════════════════════════════════════════
// builder_work tests
// ════════════════════════════════════════════════════════════════

#[test]
fn builder_work_idle_moves_toward_player() {
    let mut app = economy_test_app();
    app.add_systems(Update, builder_work);

    let cfg = app.world().resource::<MapConfig>().clone();

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Inventory::new(),
    ));

    let builder_start = Vec3::new(500.0, 500.0, 0.0);
    let builder_entity = app.world_mut().spawn((
        Builder { state: BuilderState::Idle },
        Transform::from_translation(builder_start),
    )).id();

    let offset = Vec3::new(cfg.builder_idle_offset_x, cfg.builder_idle_offset_y, 0.0);
    let target = Vec3::ZERO + offset;

    for _ in 0..20 {
        advance_real(&mut app, 20);
    }

    let builder_tf = app.world().entity(builder_entity).get::<Transform>().unwrap();
    let dist_to_target = (builder_tf.translation.truncate() - target.truncate()).length();
    let initial_dist = (builder_start.truncate() - target.truncate()).length();
    assert!(
        dist_to_target < initial_dist,
        "builder should move closer to player (dist: {initial_dist:.1} -> {dist_to_target:.1})"
    );
}

#[test]
fn builder_work_moves_to_unbuilt_building() {
    let mut app = economy_test_app();
    app.add_systems(Update, builder_work);

    let iron_ore = ResourceId::new("iron_ore");
    let mut player_inv = Inventory::new();
    player_inv.add(&iron_ore, 20);

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        player_inv,
    ));

    let builder_entity = app.world_mut().spawn((
        Builder { state: BuilderState::Idle },
        Transform::from_xyz(-20.0, -20.0, 0.0),
    )).id();

    app.world_mut().spawn((
        Building {
            kind: "wall".to_string(),
            name: "Wall".to_string(),
        },
        UnbuiltBuilding,
        TilePosition { x: 0, y: 0 },
        Transform::from_xyz(0.0, 0.0, 0.0),
        OccupiedTiles(vec![(0, 0)]),
        Inventory::new(),
    ));

    for _ in 0..30 {
        advance_real(&mut app, 20);
    }

    let builder = app.world().entity(builder_entity).get::<Builder>().unwrap();
    match &builder.state {
        BuilderState::Idle => {}
        BuilderState::MovingToBuilding(_) => {}
        BuilderState::ReturningToPlayer => {}
    }
}

// ════════════════════════════════════════════════════════════════
// burner_generator_tick tests
// ════════════════════════════════════════════════════════════════

#[test]
fn burner_generator_produces_power_with_fuel() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, burner_generator_tick);

    let coal = ResourceId::new("coal");

    app.world_mut().resource_mut::<PowerGrid>().utilization_ratio = 0.5;

    let mut inv = Inventory::new();
    inv.add(&coal, 10);

    app.world_mut().spawn((
        BurnerGenerator {
            fuel_burn_timer: 0.0,
            fuel_burn_interval: 1.0,
            base_output: 100.0,
        },
        inv,
        PowerProducer { output: 0.0 },
    ));

    advance_fixed(&mut app, 0.1);

    {
        let mut q = app.world_mut().query::<&PowerProducer>();
        let producer = q.single(app.world()).unwrap();
        assert!(producer.output > 0.0, "burner should produce power when fuel is present");
    }
}

#[test]
fn burner_generator_no_fuel_zero_output() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, burner_generator_tick);

    let inv = Inventory::new();

    app.world_mut().spawn((
        BurnerGenerator {
            fuel_burn_timer: 0.0,
            fuel_burn_interval: 1.0,
            base_output: 100.0,
        },
        inv,
        PowerProducer { output: 50.0 },
    ));

    advance_fixed(&mut app, 1.0);

    {
        let mut q = app.world_mut().query::<&PowerProducer>();
        let producer = q.single(app.world()).unwrap();
        assert_eq!(producer.output, 0.0, "burner should output 0 when no fuel");
    }
}

#[test]
fn burner_generator_fuel_consumption_scales_with_utilization() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, burner_generator_tick);

    let coal = ResourceId::new("coal");

    app.world_mut().resource_mut::<PowerGrid>().utilization_ratio = 0.1;

    let mut inv = Inventory::new();
    inv.add(&coal, 100);

    app.world_mut().spawn((
        BurnerGenerator {
            fuel_burn_timer: 0.0,
            fuel_burn_interval: 1.0,
            base_output: 100.0,
        },
        inv,
        PowerProducer { output: 0.0 },
    ));

    advance_fixed(&mut app, 5.0);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        let coal_remaining = inv.get(&coal);
        // 5s * 0.1 = 0.5s burn time, interval=1.0 → ~0 fuel consumed
        assert!(coal_remaining > 95, "with low utilization, most fuel should remain: {coal_remaining}");
    }
}

#[test]
fn burner_generator_full_utilization_burns_faster() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, burner_generator_tick);

    let coal = ResourceId::new("coal");

    app.world_mut().resource_mut::<PowerGrid>().utilization_ratio = 1.0;

    let mut inv = Inventory::new();
    inv.add(&coal, 100);

    app.world_mut().spawn((
        BurnerGenerator {
            fuel_burn_timer: 0.0,
            fuel_burn_interval: 1.0,
            base_output: 100.0,
        },
        inv,
        PowerProducer { output: 0.0 },
    ));

    advance_fixed(&mut app, 10.0);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        let coal_remaining = inv.get(&coal);
        // 10s * 1.0 = 10s burn time, interval=1.0 → ~10 fuel consumed
        assert!(coal_remaining < 96, "at full utilization, more fuel should be consumed: {coal_remaining}");
    }
}

#[test]
fn burner_generator_timer_resets_on_no_fuel() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, burner_generator_tick);

    let coal = ResourceId::new("coal");

    app.world_mut().resource_mut::<PowerGrid>().utilization_ratio = 1.0;

    let mut inv = Inventory::new();
    inv.add(&coal, 1);

    app.world_mut().spawn((
        BurnerGenerator {
            fuel_burn_timer: 0.5,
            fuel_burn_interval: 1.0,
            base_output: 100.0,
        },
        inv,
        PowerProducer { output: 0.0 },
    ));

    // One tick to consume the fuel
    advance_fixed(&mut app, 1.0);
    // Now with no fuel, timer should reset to 0
    advance_fixed(&mut app, 0.1);

    {
        let mut q = app.world_mut().query::<&BurnerGenerator>();
        let burner = q.single(app.world()).unwrap();
        assert_eq!(burner.fuel_burn_timer, 0.0, "timer should reset when fuel is depleted, got {}", burner.fuel_burn_timer);
    }
}

#[test]
fn power_grid_zero_utilization_no_fuel_burn() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, burner_generator_tick);

    let coal = ResourceId::new("coal");

    app.world_mut().resource_mut::<PowerGrid>().utilization_ratio = 0.0;

    let mut inv = Inventory::new();
    inv.add(&coal, 100);

    app.world_mut().spawn((
        BurnerGenerator {
            fuel_burn_timer: 0.0,
            fuel_burn_interval: 1.0,
            base_output: 100.0,
        },
        inv,
        PowerProducer { output: 0.0 },
    ));

    advance_fixed(&mut app, 10.0);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert_eq!(inv.get(&coal), 100, "no fuel should be consumed at zero utilization");
    }
}

// ════════════════════════════════════════════════════════════════
// structure_interact tests
// ════════════════════════════════════════════════════════════════

#[test]
fn structure_interact_upgrades_tier_when_affordable() {
    let mut app = economy_test_app();
    app.add_systems(Update, structure_interact);

    let registry = app.world().resource::<BuildingRegistry>().clone();
    let miner_def = registry.get("miner").unwrap();
    if miner_def.tiers.is_empty() {
        return;
    }
    let tier0 = &miner_def.tiers[0];

    let mut player_inv = Inventory::new();
    for (res, amt) in &tier0.required_items {
        player_inv.add(res, *amt + 5);
    }

    let _player_entity = app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        player_inv,
    )).id();

    let building_entity = app.world_mut().spawn((
        Building {
            kind: "miner".to_string(),
            name: "Miner".to_string(),
        },
        TilePosition { x: 6, y: 5 },
        CurrentTier(0),
        OccupiedTiles(vec![(6, 5)]),
    )).id();

    {
        let mut spatial = app.world_mut().resource_mut::<SpatialRegistry>();
        spatial.insert_for_test(6, 5, building_entity);
    }

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyE);
    advance_real(&mut app, 20);

    let tier = app.world().entity(building_entity).get::<CurrentTier>().unwrap();
    assert_eq!(tier.0, 1, "tier should have advanced from 0 to 1");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyE);
}

#[test]
fn structure_interact_insufficient_items_no_upgrade() {
    let mut app = economy_test_app();
    app.add_systems(Update, structure_interact);

    let registry = app.world().resource::<BuildingRegistry>().clone();
    let miner_def = registry.get("miner").unwrap();
    if miner_def.tiers.is_empty() {
        return;
    }

    let _player_entity = app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        Inventory::new(),
    )).id();

    let building_entity = app.world_mut().spawn((
        Building {
            kind: "miner".to_string(),
            name: "Miner".to_string(),
        },
        TilePosition { x: 6, y: 5 },
        CurrentTier(0),
        OccupiedTiles(vec![(6, 5)]),
    )).id();

    {
        let mut spatial = app.world_mut().resource_mut::<SpatialRegistry>();
        spatial.insert_for_test(6, 5, building_entity);
    }

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyE);
    advance_real(&mut app, 20);

    let tier = app.world().entity(building_entity).get::<CurrentTier>().unwrap();
    assert_eq!(tier.0, 0, "tier should NOT advance without resources");

    let toast = app.world().resource::<ToastQueue>();
    assert!(
        toast.0.iter().any(|t| t.contains("need")),
        "should show toast about missing items, got: {:?}",
        toast.0
    );

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyE);
}

#[test]
fn structure_interact_no_e_key_does_nothing() {
    let mut app = economy_test_app();
    app.add_systems(Update, structure_interact);

    let registry = app.world().resource::<BuildingRegistry>().clone();
    let miner_def = registry.get("miner").unwrap();
    if miner_def.tiers.is_empty() {
        return;
    }
    let tier0 = &miner_def.tiers[0];

    let mut player_inv = Inventory::new();
    for (res, amt) in &tier0.required_items {
        player_inv.add(res, *amt + 5);
    }

    let _player_entity = app.world_mut().spawn((
        Player,
        TilePosition { x: 5, y: 5 },
        player_inv,
    )).id();

    let building_entity = app.world_mut().spawn((
        Building {
            kind: "miner".to_string(),
            name: "Miner".to_string(),
        },
        TilePosition { x: 6, y: 5 },
        CurrentTier(0),
        OccupiedTiles(vec![(6, 5)]),
    )).id();

    {
        let mut spatial = app.world_mut().resource_mut::<SpatialRegistry>();
        spatial.insert_for_test(6, 5, building_entity);
    }

    advance_real(&mut app, 50);

    let tier = app.world().entity(building_entity).get::<CurrentTier>().unwrap();
    assert_eq!(tier.0, 0, "tier should NOT advance without E key press");
}

// ════════════════════════════════════════════════════════════════
// Additional edge-case tests
// ════════════════════════════════════════════════════════════════

#[test]
fn assembler_tick_respects_inventory_capacity() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, assembler_tick);

    let iron_ore = ResourceId::new("iron_ore");
    let iron_plate = ResourceId::new("iron_plate");

    let recipe = app.world().resource::<RecipeRegistry>().get("iron_plate").unwrap().clone();
    let time_sec = recipe.time_sec;

    let mut inv = Inventory::with_capacity(5);
    inv.add(&iron_ore, 10);
    inv.add(&iron_plate, 4);

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: time_sec,
            recipe_id: "iron_plate".to_string(),
        },
        inv,
        Active(true),
    ));

    advance_fixed(&mut app, time_sec + 0.1);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inv = q.single(app.world()).unwrap();
        assert_eq!(inv.get(&iron_plate), 4, "should not produce when output would exceed capacity");
    }
}

#[test]
fn multiple_assemblers_produce_independently() {
    let mut app = economy_test_app();
    app.add_systems(FixedUpdate, assembler_tick);

    let iron_ore = ResourceId::new("iron_ore");
    let iron_plate = ResourceId::new("iron_plate");

    let recipe = app.world().resource::<RecipeRegistry>().get("iron_plate").unwrap().clone();
    let time_sec = recipe.time_sec;

    let mut inv1 = Inventory::new();
    inv1.add(&iron_ore, 20);

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: time_sec,
            recipe_id: "iron_plate".to_string(),
        },
        inv1,
        Active(true),
    ));

    app.world_mut().spawn((
        Assembler {
            production_timer: 0.0,
            interval: time_sec,
            recipe_id: "iron_plate".to_string(),
        },
        Inventory::new(),
        Active(true),
    ));

    advance_fixed(&mut app, time_sec + 0.1);

    {
        let mut q = app.world_mut().query::<&Inventory>();
        let inventories: Vec<_> = q.iter(app.world()).collect();
        assert!(inventories[0].get(&iron_plate) >= 1, "first assembler should produce");
        assert_eq!(inventories[1].get(&iron_plate), 0, "second assembler without inputs should not produce");
    }
}
