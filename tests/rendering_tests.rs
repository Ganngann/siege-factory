use bevy::prelude::*;
use siege_factory::economy::components::{HasHpBar, HpBarChild};
use siege_factory::enemy::components::Health;
use siege_factory::map::config::MapConfig;
use siege_factory::rendering::cache::TextureCache;
use siege_factory::rendering::config::VisualsConfig;
use siege_factory::rendering::visuals::*;

fn rendering_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(MapConfig::load());
    app.insert_resource(VisualsConfig::load());
    app
}// ════════════════════════════════════════════════════════════════
// ensure_hp_bars
// ════════════════════════════════════════════════════════════════

#[test]
fn ensure_hp_bars_adds_to_new_entities() {
    let mut app = rendering_test_app();
    app.add_systems(Update, ensure_hp_bars);

    app.world_mut()
        .spawn(Health {
            current: 100,
            max: 100,
        });

    app.update();

    let has_hp_bar = app
        .world_mut()
        .query::<&HasHpBar>()
        .iter(app.world())
        .count();
    assert_eq!(has_hp_bar, 1);

    let hp_bar_children = app
        .world_mut()
        .query::<&HpBarChild>()
        .iter(app.world())
        .count();
    assert_eq!(hp_bar_children, 1);
}

#[test]
fn ensure_hp_bars_skips_already_marked() {
    let mut app = rendering_test_app();
    app.add_systems(Update, ensure_hp_bars);

    // Spawn entity with Health and HasHpBar already
    app.world_mut().spawn((
        Health {
            current: 100,
            max: 100,
        },
        HasHpBar,
    ));

    app.update();

    // No HpBarChild should be spawned
    let hp_bar_children = app
        .world_mut()
        .query::<&HpBarChild>()
        .iter(app.world())
        .count();
    assert_eq!(hp_bar_children, 0);
}

#[test]
fn ensure_hp_bars_multiple_entities() {
    let mut app = rendering_test_app();
    app.add_systems(Update, ensure_hp_bars);

    app.world_mut()
        .spawn(Health {
            current: 50,
            max: 100,
        });
    app.world_mut()
        .spawn(Health {
            current: 75,
            max: 100,
        });
    app.world_mut()
        .spawn(Health {
            current: 25,
            max: 100,
        });

    app.update();

    let has_hp_bar = app
        .world_mut()
        .query::<&HasHpBar>()
        .iter(app.world())
        .count();
    assert_eq!(has_hp_bar, 3);

    let hp_bar_children = app
        .world_mut()
        .query::<&HpBarChild>()
        .iter(app.world())
        .count();
    assert_eq!(hp_bar_children, 3);
}

#[test]
fn ensure_hp_bars_child_has_correct_components() {
    let mut app = rendering_test_app();
    app.add_systems(Update, ensure_hp_bars);

    let parent = app
        .world_mut()
        .spawn(Health {
            current: 100,
            max: 100,
        })
        .id();

    app.update();

    // Check parent has Children
    let children = app.world().entity(parent).get::<Children>().unwrap();
    assert_eq!(children.len(), 1);

    // Check child has HpBarChild and Sprite
    let child_entity = children[0];
    let child = app.world().entity(child_entity);
    assert!(child.get::<HpBarChild>().is_some());
    assert!(child.get::<Sprite>().is_some());
    assert!(child.get::<Transform>().is_some());
}

// ════════════════════════════════════════════════════════════════
// update_hp_bars
// ════════════════════════════════════════════════════════════════

#[test]
fn update_hp_bars_changes_width_for_half_hp() {
    let mut app = rendering_test_app();
    let hp_bar_width = app.world().resource::<VisualsConfig>().hp_bar.width;
    let hp_bar_height = app.world().resource::<VisualsConfig>().hp_bar.height;
    app.add_systems(Update, update_hp_bars);

    let parent = app
        .world_mut()
        .spawn(Health {
            current: 50,
            max: 100,
        })
        .id();

    let child = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width, hp_bar_height)),
                ..default()
            },
        ))
        .id();

    app.world_mut().entity_mut(parent).add_child(child);

    app.update();

    let sprite = app.world().entity(child).get::<Sprite>().unwrap();
    let size = sprite.custom_size.unwrap();
    // 50/100 = 0.5, so width should be hp_bar_width * 0.5
    assert!((size.x - hp_bar_width * 0.5).abs() < 0.01);
    assert!((size.y - hp_bar_height).abs() < 0.01);
}

#[test]
fn update_hp_bars_changes_width_for_full_hp() {
    let mut app = rendering_test_app();
    let hp_bar_width = app.world().resource::<VisualsConfig>().hp_bar.width;
    let hp_bar_height = app.world().resource::<VisualsConfig>().hp_bar.height;
    app.add_systems(Update, update_hp_bars);

    let parent = app
        .world_mut()
        .spawn(Health {
            current: 100,
            max: 100,
        })
        .id();

    let child = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width * 0.5, hp_bar_height)),
                ..default()
            },
        ))
        .id();

    app.world_mut().entity_mut(parent).add_child(child);

    app.update();

    let sprite = app.world().entity(child).get::<Sprite>().unwrap();
    let size = sprite.custom_size.unwrap();
    // 100/100 = 1.0, so width should be hp_bar_width
    assert!((size.x - hp_bar_width).abs() < 0.01);
}

#[test]
fn update_hp_bars_changes_width_for_zero_hp() {
    let mut app = rendering_test_app();
    let hp_bar_width = app.world().resource::<VisualsConfig>().hp_bar.width;
    let hp_bar_height = app.world().resource::<VisualsConfig>().hp_bar.height;
    app.add_systems(Update, update_hp_bars);

    let parent = app
        .world_mut()
        .spawn(Health {
            current: 0,
            max: 100,
        })
        .id();

    let child = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width, hp_bar_height)),
                ..default()
            },
        ))
        .id();

    app.world_mut().entity_mut(parent).add_child(child);

    app.update();

    let sprite = app.world().entity(child).get::<Sprite>().unwrap();
    let size = sprite.custom_size.unwrap();
    // 0/100 = 0.0, so width should be 0
    assert!(size.x.abs() < 0.01);
}

#[test]
fn update_hp_bars_changes_color_high_hp() {
    let mut app = rendering_test_app();
    let hp_bar_width = app.world().resource::<VisualsConfig>().hp_bar.width;
    let color_high = app.world().resource::<VisualsConfig>().hp_bar.color_high;
    app.add_systems(Update, update_hp_bars);

    let parent = app
        .world_mut()
        .spawn(Health {
            current: 100,
            max: 100,
        })
        .id();

    let child = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width, 3.0)),
                color: Color::srgb(0.0, 0.0, 0.0),
                ..default()
            },
        ))
        .id();

    app.world_mut().entity_mut(parent).add_child(child);

    app.update();

    let sprite = app.world().entity(child).get::<Sprite>().unwrap();
    let Color::Srgba(actual) = sprite.color else {
        panic!("expected Srgba")
    };
    let Color::Srgba(expected) = color_high else {
        panic!("expected Srgba")
    };
    assert!((actual.red - expected.red).abs() < 0.01);
    assert!((actual.green - expected.green).abs() < 0.01);
    assert!((actual.blue - expected.blue).abs() < 0.01);
}

#[test]
fn update_hp_bars_changes_color_low_hp() {
    let mut app = rendering_test_app();
    let hp_bar_width = app.world().resource::<VisualsConfig>().hp_bar.width;
    let color_low = app.world().resource::<VisualsConfig>().hp_bar.color_low;
    app.add_systems(Update, update_hp_bars);

    let parent = app
        .world_mut()
        .spawn(Health {
            current: 10,
            max: 100,
        })
        .id();

    let child = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width, 3.0)),
                color: Color::srgb(0.0, 1.0, 0.0),
                ..default()
            },
        ))
        .id();

    app.world_mut().entity_mut(parent).add_child(child);

    app.update();

    let sprite = app.world().entity(child).get::<Sprite>().unwrap();
    let Color::Srgba(actual) = sprite.color else {
        panic!("expected Srgba")
    };
    let Color::Srgba(expected) = color_low else {
        panic!("expected Srgba")
    };
    assert!((actual.red - expected.red).abs() < 0.01);
    assert!((actual.green - expected.green).abs() < 0.01);
    assert!((actual.blue - expected.blue).abs() < 0.01);
}

#[test]
fn update_hp_bars_changes_color_mid_hp() {
    let mut app = rendering_test_app();
    let hp_bar_width = app.world().resource::<VisualsConfig>().hp_bar.width;
    let color_mid = app.world().resource::<VisualsConfig>().hp_bar.color_mid;
    app.add_systems(Update, update_hp_bars);

    let parent = app
        .world_mut()
        .spawn(Health {
            current: 40,
            max: 100,
        })
        .id();

    let child = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width, 3.0)),
                color: Color::srgb(0.0, 0.0, 0.0),
                ..default()
            },
        ))
        .id();

    app.world_mut().entity_mut(parent).add_child(child);

    app.update();

    let sprite = app.world().entity(child).get::<Sprite>().unwrap();
    let Color::Srgba(actual) = sprite.color else {
        panic!("expected Srgba")
    };
    let Color::Srgba(expected) = color_mid else {
        panic!("expected Srgba")
    };
    assert!((actual.red - expected.red).abs() < 0.01);
    assert!((actual.green - expected.green).abs() < 0.01);
    assert!((actual.blue - expected.blue).abs() < 0.01);
}

#[test]
fn update_hp_bars_multiple_entities() {
    let mut app = rendering_test_app();
    let hp_bar_width = app.world().resource::<VisualsConfig>().hp_bar.width;
    let hp_bar_height = app.world().resource::<VisualsConfig>().hp_bar.height;
    app.add_systems(Update, update_hp_bars);

    // Entity 1: 75% HP
    let p1 = app
        .world_mut()
        .spawn(Health {
            current: 75,
            max: 100,
        })
        .id();
    let c1 = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width, hp_bar_height)),
                ..default()
            },
        ))
        .id();
    app.world_mut().entity_mut(p1).add_child(c1);

    // Entity 2: 25% HP
    let p2 = app
        .world_mut()
        .spawn(Health {
            current: 25,
            max: 100,
        })
        .id();
    let c2 = app
        .world_mut()
        .spawn((
            HpBarChild,
            Sprite {
                custom_size: Some(Vec2::new(hp_bar_width, hp_bar_height)),
                ..default()
            },
        ))
        .id();
    app.world_mut().entity_mut(p2).add_child(c2);

    app.update();

    let s1 = app.world().entity(c1).get::<Sprite>().unwrap();
    let s2 = app.world().entity(c2).get::<Sprite>().unwrap();

    assert!((s1.custom_size.unwrap().x - hp_bar_width * 0.75).abs() < 0.01);
    assert!((s2.custom_size.unwrap().x - hp_bar_width * 0.25).abs() < 0.01);
}

// ════════════════════════════════════════════════════════════════
// direction_arrow
// ════════════════════════════════════════════════════════════════

#[test]
fn direction_arrow_returns_correct_symbols() {
    use siege_factory::economy::components::Direction;

    assert_eq!(direction_arrow(Direction::East), ">");
    assert_eq!(direction_arrow(Direction::North), "^");
    assert_eq!(direction_arrow(Direction::West), "<");
    assert_eq!(direction_arrow(Direction::South), "v");
}

// ════════════════════════════════════════════════════════════════
// TileHighlightEntity resource
// ════════════════════════════════════════════════════════════════

#[test]
fn tile_highlight_entity_default_is_none() {
    let entity = TileHighlightEntity::default();
    assert!(entity.0.is_none());
}

// ════════════════════════════════════════════════════════════════
// VisualsConfig section access
// ════════════════════════════════════════════════════════════════

#[test]
fn visuals_config_hp_bar_width_positive() {
    let cfg = VisualsConfig::load();
    assert!(cfg.hp_bar.width > 0.0);
}

#[test]
fn visuals_config_projectile_scale_positive() {
    let cfg = VisualsConfig::load();
    assert!(cfg.projectile.scale > 0.0);
}

#[test]
fn visuals_config_belt_item_z_positive() {
    let cfg = VisualsConfig::load();
    assert!(cfg.belt_item.z > 0.0);
}

#[test]
fn visuals_config_unit_dimensions_positive() {
    let cfg = VisualsConfig::load();
    assert!(cfg.unit.width > 0.0);
    assert!(cfg.unit.height > 0.0);
}

#[test]
fn visuals_config_enemy_sizes_are_distinct() {
    let cfg = VisualsConfig::load();
    assert!(cfg.enemy.boss_size != cfg.enemy.tank_size);
    assert!(cfg.enemy.tank_size != cfg.enemy.default_size);
}

// ════════════════════════════════════════════════════════════════
// attach_enemy_visuals
// ════════════════════════════════════════════════════════════════

#[test]
fn attach_enemy_visuals_adds_sprite() {
    let mut app = rendering_test_app();
    app.init_resource::<TextureCache>();
    app.add_systems(Update, attach_enemy_visuals);

    app.world_mut().spawn(siege_factory::enemy::components::Enemy {
        kind: "basic".to_string(),
    });

    app.update();

    let count = app
        .world_mut()
        .query::<&Sprite>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn attach_enemy_visuals_skips_already_visualized() {
    let mut app = rendering_test_app();
    app.init_resource::<TextureCache>();
    app.add_systems(Update, attach_enemy_visuals);

    app.world_mut().spawn((
        siege_factory::enemy::components::Enemy {
            kind: "basic".to_string(),
        },
        Sprite::default(),
    ));

    app.update();

    // Should not add a second sprite (the default one)
    let count = app
        .world_mut()
        .query::<&Sprite>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}
