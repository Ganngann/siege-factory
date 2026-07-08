use bevy::prelude::*;
use siege_factory::core::utils::{
    config_dir, move_toward, parse_hex_color, tile_to_world, tile_to_world_corner, world_to_tile,
};
use siege_factory::map::config::{
    MapConfig, PlacedStructure, PlacedStructureProps, StartingAreaConfig,
};
use siege_factory::map::rng::{chunk_hash, SimpleRng};
use siege_factory::core::modding::ModRegistry;


fn test_mods() -> ModRegistry { ModRegistry::for_test() }
fn assert_color_eq(c: Color, r: f32, g: f32, b: f32) {
    let Color::Srgba(srgba) = c else { panic!("expected Color::Srgba, got {:?}", c); };
    assert!(
        (srgba.red - r).abs() < 1e-6,
        "red: expected {}, got {}",
        r,
        srgba.red
    );
    assert!(
        (srgba.green - g).abs() < 1e-6,
        "green: expected {}, got {}",
        g,
        srgba.green
    );
    assert!(
        (srgba.blue - b).abs() < 1e-6,
        "blue: expected {}, got {}",
        b,
        srgba.blue
    );
}

// ── parse_hex_color ──────────────────────────────────────────────────────

#[test]
fn parse_hex_color_six_digit_with_hash() {
    assert_color_eq(parse_hex_color("#FF0000"), 1.0, 0.0, 0.0);
}

#[test]
fn parse_hex_color_six_digit_no_hash() {
    assert_color_eq(parse_hex_color("00FF00"), 0.0, 1.0, 0.0);
}

#[test]
fn parse_hex_color_white() {
    assert_color_eq(parse_hex_color("#FFFFFF"), 1.0, 1.0, 1.0);
}

#[test]
fn parse_hex_color_black() {
    assert_color_eq(parse_hex_color("#000000"), 0.0, 0.0, 0.0);
}

#[test]
fn parse_hex_color_short_three_digit_returns_grey() {
    // "F0A" -> len 3 < 6 => fallback grey
    assert_color_eq(parse_hex_color("#F0A"), 0.5, 0.5, 0.5);
}

#[test]
fn parse_hex_color_too_short_returns_grey() {
    assert_color_eq(parse_hex_color("abc"), 0.5, 0.5, 0.5);
}

#[test]
fn parse_hex_color_empty_returns_grey() {
    assert_color_eq(parse_hex_color(""), 0.5, 0.5, 0.5);
}

#[test]
fn parse_hex_color_invalid_hex_digits_returns_mid_grey() {
    // "ZZZZZZ" -> from_str_radix fails -> unwrap_or(128) -> 128/255
    let expected = 128.0 / 255.0;
    assert_color_eq(parse_hex_color("#ZZZZZZ"), expected, expected, expected);
}

#[test]
fn parse_hex_color_with_leading_whitespace_not_stripped() {
    // trim_start_matches('#') only strips '#', not spaces.
    // "  #FF0000" -> stays "  #FF0000" (len 9 >= 6), so it parses substrings
    // of the space/# chars, yielding invalid hex defaults. Just verify it doesn't panic.
    let c = parse_hex_color("  #FF0000");
    let Color::Srgba(_) = c else { panic!("expected Srgba variant"); };
}

#[test]
fn parse_hex_color_only_hash_stripped() {
    // "#FF0000" -> trim_start_matches('#') -> "FF0000"
    assert_color_eq(parse_hex_color("#FF0000"), 1.0, 0.0, 0.0);
}

#[test]
fn parse_hex_color_mid_values() {
    let expected = 128.0 / 255.0;
    assert_color_eq(parse_hex_color("#808080"), expected, expected, expected);
}

#[test]
fn parse_hex_color_only_hash_returns_grey() {
    assert_color_eq(parse_hex_color("#"), 0.5, 0.5, 0.5);
}

#[test]
fn parse_hex_color_five_chars_returns_grey() {
    assert_color_eq(parse_hex_color("12345"), 0.5, 0.5, 0.5);
}

#[test]
fn parse_hex_color_mixed_case() {
    assert_color_eq(parse_hex_color("#aAbBcC"), 0xaa as f32 / 255.0, 0xbb as f32 / 255.0, 0xcc as f32 / 255.0);
}

// ── tile_to_world ────────────────────────────────────────────────────────

#[test]
fn tile_to_world_origin() {
    let v = tile_to_world(0, 0, 32.0);
    assert!((v.x - 0.0).abs() < 1e-6);
    assert!((v.y - 0.0).abs() < 1e-6);
}

#[test]
fn tile_to_world_positive() {
    let v = tile_to_world(3, 5, 32.0);
    assert!((v.x - 96.0).abs() < 1e-6);
    assert!((v.y - 160.0).abs() < 1e-6);
}

#[test]
fn tile_to_world_negative() {
    let v = tile_to_world(-2, -1, 16.0);
    assert!((v.x - (-32.0)).abs() < 1e-6);
    assert!((v.y - (-16.0)).abs() < 1e-6);
}

#[test]
fn tile_to_world_zero_tile_size() {
    let v = tile_to_world(5, 5, 0.0);
    assert!((v.x - 0.0).abs() < 1e-6);
    assert!((v.y - 0.0).abs() < 1e-6);
}

#[test]
fn tile_to_world_large_tile_size() {
    let v = tile_to_world(1, 1, 1000.0);
    assert!((v.x - 1000.0).abs() < 1e-6);
    assert!((v.y - 1000.0).abs() < 1e-6);
}

#[test]
fn tile_to_world_formula_check() {
    let ts = 32.0;
    let (tx, ty) = (7, 11);
    let v = tile_to_world(tx, ty, ts);
    assert!((v.x - tx as f32 * ts).abs() < 1e-6);
    assert!((v.y - ty as f32 * ts).abs() < 1e-6);
}

#[test]
fn tile_to_world_large_negative() {
    let v = tile_to_world(-1000, -1000, 32.0);
    assert!((v.x - (-32000.0)).abs() < 1e-6);
    assert!((v.y - (-32000.0)).abs() < 1e-6);
}

#[test]
fn tile_to_world_one_by_one() {
    let v = tile_to_world(1, 1, 1.0);
    assert!((v.x - 1.0).abs() < 1e-6);
    assert!((v.y - 1.0).abs() < 1e-6);
}

// ── tile_to_world_corner ─────────────────────────────────────────────────

#[test]
fn tile_to_world_corner_origin() {
    let v = tile_to_world_corner(0, 0, 32.0);
    assert!((v.x - (-16.0)).abs() < 1e-6);
    assert!((v.y - (-16.0)).abs() < 1e-6);
}

#[test]
fn tile_to_world_corner_positive() {
    let v = tile_to_world_corner(3, 5, 32.0);
    assert!((v.x - 80.0).abs() < 1e-6);
    assert!((v.y - 144.0).abs() < 1e-6);
}

#[test]
fn tile_to_world_corner_negative_tile() {
    let v = tile_to_world_corner(-1, -1, 32.0);
    assert!((v.x - (-48.0)).abs() < 1e-6);
    assert!((v.y - (-48.0)).abs() < 1e-6);
}

#[test]
fn tile_to_world_corner_formula_check() {
    let ts = 16.0;
    let (tx, ty) = (4, 7);
    let v = tile_to_world_corner(tx, ty, ts);
    assert!((v.x - (tx as f32 * ts - ts / 2.0)).abs() < 1e-6);
    assert!((v.y - (ty as f32 * ts - ts / 2.0)).abs() < 1e-6);
}

#[test]
fn tile_to_world_corner_offset_from_center() {
    let ts = 64.0;
    let center = tile_to_world(2, 3, ts);
    let corner = tile_to_world_corner(2, 3, ts);
    assert!((center.x - corner.x - ts / 2.0).abs() < 1e-6);
    assert!((center.y - corner.y - ts / 2.0).abs() < 1e-6);
}

#[test]
fn tile_to_world_corner_zero_tile_size() {
    let v = tile_to_world_corner(5, 5, 0.0);
    assert!((v.x - 0.0).abs() < 1e-6);
    assert!((v.y - 0.0).abs() < 1e-6);
}

// ── world_to_tile ────────────────────────────────────────────────────────

#[test]
fn world_to_tile_at_center_of_tile() {
    let (tx, ty) = world_to_tile(Vec2::new(64.0, 96.0), 32.0);
    assert_eq!(tx, 2);
    assert_eq!(ty, 3);
}

#[test]
fn world_to_tile_at_origin() {
    let (tx, ty) = world_to_tile(Vec2::new(0.0, 0.0), 32.0);
    assert_eq!(tx, 0);
    assert_eq!(ty, 0);
}

#[test]
fn world_to_tile_negative_world_pos() {
    let (tx, ty) = world_to_tile(Vec2::new(-32.0, -32.0), 32.0);
    assert_eq!(tx, -1);
    assert_eq!(ty, -1);
}

#[test]
fn world_to_tile_boundary_left_edge() {
    let (tx, _) = world_to_tile(Vec2::new(-16.0, 0.0), 32.0);
    assert_eq!(tx, 0);
}

#[test]
fn world_to_tile_boundary_right_edge() {
    let (tx, _) = world_to_tile(Vec2::new(15.999, 0.0), 32.0);
    assert_eq!(tx, 0);
}

#[test]
fn world_to_tile_just_past_right_edge() {
    let (tx, _) = world_to_tile(Vec2::new(16.001, 0.0), 32.0);
    assert_eq!(tx, 1);
}

#[test]
fn world_to_tile_large_world_pos() {
    // ((10000 + 16) / 32).floor() = floor(313.0) = 313
    let (tx, ty) = world_to_tile(Vec2::new(10000.0, 10000.0), 32.0);
    assert_eq!(tx, 313);
    assert_eq!(ty, 313);
}

#[test]
fn world_to_tile_small_tile_size() {
    let (tx, ty) = world_to_tile(Vec2::new(5.0, 5.0), 1.0);
    assert_eq!(tx, 5);
    assert_eq!(ty, 5);
}

// ── tile_to_world / world_to_tile round-trip ─────────────────────────────

#[test]
fn round_trip_tile_world_origin() {
    let ts = 32.0;
    let pos = tile_to_world(0, 0, ts);
    let (tx, ty) = world_to_tile(pos, ts);
    assert_eq!((tx, ty), (0, 0));
}

#[test]
fn round_trip_tile_world_positive_grid() {
    let ts = 32.0;
    for tx in 0..20 {
        for ty in 0..15 {
            let pos = tile_to_world(tx, ty, ts);
            let (rx, ry) = world_to_tile(pos, ts);
            assert_eq!((rx, ry), (tx, ty), "failed round-trip for tile ({tx}, {ty})");
        }
    }
}

#[test]
fn round_trip_tile_world_negative_grid() {
    let ts = 32.0;
    for tx in -10..0 {
        for ty in -10..0 {
            let pos = tile_to_world(tx, ty, ts);
            let (rx, ry) = world_to_tile(pos, ts);
            assert_eq!((rx, ry), (tx, ty), "failed round-trip for tile ({tx}, {ty})");
        }
    }
}

#[test]
fn round_trip_tile_world_mixed() {
    let ts = 16.0;
    let tiles: &[(i32, i32)] = &[
        (0, 0),
        (1, 0),
        (0, 1),
        (-1, -1),
        (100, 50),
        (-50, 100),
        (999, -999),
    ];
    for &(tx, ty) in tiles {
        let pos = tile_to_world(tx, ty, ts);
        let (rx, ry) = world_to_tile(pos, ts);
        assert_eq!((rx, ry), (tx, ty), "failed round-trip for tile ({tx}, {ty})");
    }
}

#[test]
fn round_trip_tile_world_various_tile_sizes() {
    let sizes: &[f32] = &[1.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0];
    for &ts in sizes {
        for tx in -5..=5 {
            for ty in -5..=5 {
                let pos = tile_to_world(tx, ty, ts);
                let (rx, ry) = world_to_tile(pos, ts);
                assert_eq!(
                    (rx, ry),
                    (tx, ty),
                    "failed round-trip for tile ({tx}, {ty}) with ts={ts}"
                );
            }
        }
    }
}

// ── move_toward ──────────────────────────────────────────────────────────

#[test]
fn move_toward_already_at_target() {
    let mut pos = Vec3::new(10.0, 20.0, 0.0);
    let target = Vec3::new(10.0, 20.0, 0.0);
    let arrived = move_toward(&mut pos, target, 100.0, 1.0);
    assert!(arrived);
    assert!((pos.x - 10.0).abs() < 1e-6);
    assert!((pos.y - 20.0).abs() < 1e-6);
}

#[test]
fn move_toward_very_close_returns_arrived() {
    let mut pos = Vec3::new(10.0, 20.0, 0.0);
    let target = Vec3::new(10.0001, 20.0, 0.0);
    let arrived = move_toward(&mut pos, target, 100.0, 1.0);
    assert!(arrived);
}

#[test]
fn move_toward_steps_toward_target() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(100.0, 0.0, 0.0);
    let arrived = move_toward(&mut pos, target, 10.0, 1.0);
    assert!(!arrived);
    assert!((pos.x - 10.0).abs() < 1e-6);
    assert!((pos.y - 0.0).abs() < 1e-6);
}

#[test]
fn move_toward_diagonal_exact_arrival() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(3.0, 4.0, 0.0);
    // step = min(5.0, 5.0) = 5.0 = dist, but returns false on the move call
    let arrived = move_toward(&mut pos, target, 5.0, 1.0);
    assert!(!arrived);
    assert!((pos.x - 3.0).abs() < 1e-6);
    assert!((pos.y - 4.0).abs() < 1e-6);
    // Now at target, next call detects arrival
    let arrived = move_toward(&mut pos, target, 5.0, 1.0);
    assert!(arrived);
}

#[test]
fn move_toward_partial_step() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(100.0, 0.0, 0.0);
    let arrived = move_toward(&mut pos, target, 10.0, 1.0);
    assert!(!arrived);
    assert!((pos.x - 10.0).abs() < 1e-6);
}

#[test]
fn move_toward_step_clamped_to_dist() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(5.0, 0.0, 0.0);
    // speed * dt = 100, dist = 5 => step = min(100, 5) = 5
    // moves to target but returns false; arrival detected on next call
    let arrived = move_toward(&mut pos, target, 100.0, 1.0);
    assert!(!arrived);
    assert!((pos.x - 5.0).abs() < 1e-6);
    let arrived = move_toward(&mut pos, target, 100.0, 1.0);
    assert!(arrived);
}

#[test]
fn move_toward_y_axis() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(0.0, 50.0, 0.0);
    let arrived = move_toward(&mut pos, target, 25.0, 1.0);
    assert!(!arrived);
    assert!((pos.y - 25.0).abs() < 1e-6);
    assert!((pos.x - 0.0).abs() < 1e-6);
}

#[test]
fn move_toward_negative_direction() {
    let mut pos = Vec3::new(10.0, 10.0, 0.0);
    let target = Vec3::new(0.0, 0.0, 0.0);
    let arrived = move_toward(&mut pos, target, 10.0, 1.0);
    assert!(!arrived);
    assert!(pos.x < 10.0);
    assert!(pos.y < 10.0);
}

#[test]
fn move_toward_z_ignored() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(0.0, 0.0, 100.0);
    let arrived = move_toward(&mut pos, target, 50.0, 1.0);
    assert!(arrived);
    assert!((pos.z - 0.0).abs() < 1e-6);
}

#[test]
fn move_toward_small_dt() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(1000.0, 0.0, 0.0);
    let arrived = move_toward(&mut pos, target, 100.0, 0.001);
    assert!(!arrived);
    assert!((pos.x - 0.1).abs() < 1e-6);
}

#[test]
fn move_toward_zero_speed() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(100.0, 0.0, 0.0);
    let arrived = move_toward(&mut pos, target, 0.0, 1.0);
    assert!(!arrived);
    assert!((pos.x - 0.0).abs() < 1e-6);
}

#[test]
fn move_toward_zero_dt() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(100.0, 0.0, 0.0);
    let arrived = move_toward(&mut pos, target, 100.0, 0.0);
    assert!(!arrived);
    assert!((pos.x - 0.0).abs() < 1e-6);
}

#[test]
fn move_toward_multiple_steps_converge() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(20.0, 0.0, 0.0);
    let mut arrived = false;
    for _ in 0..100 {
        arrived = move_toward(&mut pos, target, 5.0, 1.0);
        if arrived {
            break;
        }
    }
    assert!(arrived);
    assert!((pos.x - 20.0).abs() < 1e-6);
}

#[test]
fn move_toward_diagonal_magnitude_preserved() {
    let mut pos = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(3.0, 4.0, 0.0);
    let _ = move_toward(&mut pos, target, 2.5, 1.0);
    let dist = (pos.x * pos.x + pos.y * pos.y).sqrt();
    assert!((dist - 2.5).abs() < 1e-6);
}

// ── silent_despawn ───────────────────────────────────────────────────────

#[test]
fn silent_despawn_function_exists() {
    let _ = siege_factory::core::utils::silent_despawn as fn(&mut Commands, Entity);
}

// ── config_dir ───────────────────────────────────────────────────────────

#[test]
fn config_dir_returns_valid_path() {
    let dir = config_dir();
    let last = dir.file_name().unwrap().to_str().unwrap();
    assert_eq!(last, "siege-factory");
}

#[test]
fn config_dir_returns_nonempty() {
    let dir = config_dir();
    assert!(!dir.to_str().unwrap().is_empty());
}

#[test]
fn config_dir_ends_with_siege_factory() {
    let dir = config_dir();
    let s = dir.to_str().unwrap();
    assert!(
        s.ends_with("siege-factory"),
        "expected path ending with 'siege-factory', got: {s}"
    );
}

// ── SimpleRng ────────────────────────────────────────────────────────────

#[test]
fn simple_rng_deterministic() {
    let mut rng_a = SimpleRng::new(42);
    let mut rng_b = SimpleRng::new(42);
    for _ in 0..100 {
        assert_eq!(rng_a.next(), rng_b.next());
    }
}

#[test]
fn simple_rng_different_seeds_differ() {
    let mut rng_a = SimpleRng::new(1);
    let mut rng_b = SimpleRng::new(2);
    let vals_a: Vec<u32> = (0..10).map(|_| rng_a.next()).collect();
    let vals_b: Vec<u32> = (0..10).map(|_| rng_b.next()).collect();
    assert_ne!(vals_a, vals_b);
}

#[test]
fn simple_rng_no_panic_on_zero_seed() {
    let mut rng = SimpleRng::new(0);
    let _ = rng.next();
}

#[test]
fn simple_rng_no_panic_on_max_seed() {
    let mut rng = SimpleRng::new(u64::MAX);
    let _ = rng.next();
}

#[test]
fn simple_rng_consecutive_values_differ() {
    let mut rng = SimpleRng::new(42);
    let first = rng.next();
    let second = rng.next();
    assert_ne!(first, second);
}

#[test]
fn simple_rng_many_values_not_all_same() {
    let mut rng = SimpleRng::new(99);
    let vals: Vec<u32> = (0..1000).map(|_| rng.next()).collect();
    let all_same = vals.iter().all(|&v| v == vals[0]);
    assert!(!all_same);
}

#[test]
fn simple_rng_wrap_around_state() {
    let mut rng = SimpleRng::new(u64::MAX - 1);
    for _ in 0..1000 {
        let _ = rng.next();
    }
}

// ── chunk_hash ───────────────────────────────────────────────────────────

#[test]
fn chunk_hash_deterministic() {
    let a = chunk_hash(42, 3, 5);
    let b = chunk_hash(42, 3, 5);
    assert_eq!(a, b);
}

#[test]
fn chunk_hash_different_coords_differ() {
    let h1 = chunk_hash(42, 0, 0);
    let h2 = chunk_hash(42, 1, 0);
    let h3 = chunk_hash(42, 0, 1);
    assert_ne!(h1, h2);
    assert_ne!(h1, h3);
    assert_ne!(h2, h3);
}

#[test]
fn chunk_hash_different_seeds_differ() {
    let h1 = chunk_hash(1, 3, 5);
    let h2 = chunk_hash(2, 3, 5);
    assert_ne!(h1, h2);
}

#[test]
fn chunk_hash_negative_coords_deterministic() {
    let h1 = chunk_hash(42, -1, -2);
    let h2 = chunk_hash(42, -1, -2);
    assert_eq!(h1, h2);

    let h3 = chunk_hash(42, 1, 2);
    assert_ne!(h1, h3);
}

#[test]
fn chunk_hash_symmetry_is_broken() {
    let h1 = chunk_hash(42, 3, 7);
    let h2 = chunk_hash(42, 7, 3);
    assert_ne!(h1, h2);
}

#[test]
fn chunk_hash_zero_seed() {
    let h1 = chunk_hash(0, 0, 0);
    let h2 = chunk_hash(0, 0, 0);
    assert_eq!(h1, h2);
}

#[test]
fn chunk_hash_large_coords() {
    let h1 = chunk_hash(42, i32::MAX, i32::MAX);
    let h2 = chunk_hash(42, i32::MAX, i32::MAX);
    assert_eq!(h1, h2);
}

#[test]
fn chunk_hash_negative_vs_positive_differ() {
    let h1 = chunk_hash(42, -1, -1);
    let h2 = chunk_hash(42, 1, 1);
    assert_ne!(h1, h2);
}

// ── MapConfig ────────────────────────────────────────────────────────────

#[test]
fn map_config_loads_correctly() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.tile_size, 32.0);
    assert_eq!(cfg.seed, 42);
    assert_eq!(cfg.chunk_size, 32);
}

#[test]
fn map_config_deposits() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.deposit_min_amount, 30);
    assert_eq!(cfg.deposit_max_amount, 80);
    assert_eq!(cfg.deposit_spawn_chance_pct, 25);
    assert_eq!(cfg.deposit_min_per_chunk, 3);
    assert_eq!(cfg.deposit_max_per_chunk, 6);
    assert!(cfg.infinite_deposits);
}

#[test]
fn map_config_distribution_sorted_by_weight_desc() {
    let cfg = MapConfig::load(&test_mods());
    assert!(!cfg.deposit_distribution.is_empty());
    for w in cfg.deposit_distribution.windows(2) {
        assert!(w[0].1 >= w[1].1, "distribution not sorted descending");
    }
}

#[test]
fn map_config_player() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.player_hp, 100);
    assert_eq!(cfg.player_speed, 250.0);
    assert_eq!(cfg.player_start_position, (5, 5));
}

#[test]
fn map_config_chunk_margins() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.initial_margin, 3);
    assert_eq!(cfg.despawn_margin, 3);
}

#[test]
fn map_config_resource_discovery_map() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(
        cfg.resource_discovery_map.get("scrap_metal").unwrap(),
        "mine_scrap_metal"
    );
    assert_eq!(
        cfg.resource_discovery_map.get("wood").unwrap(),
        "mine_wood"
    );
    assert_eq!(
        cfg.resource_discovery_map.get("stone").unwrap(),
        "mine_stone"
    );
}

/// Parse the effective `[starting_area]` from mod data, in the same merge order
/// that `MapConfig::load` uses (last enabled mod with [starting_area] wins).
/// Returns the parsed structure entries as TOML Values.
fn effective_starting_area_raw(mods: &ModRegistry) -> Vec<toml::Value> {
    mods.load_all_data("map_config.toml")
        .into_iter()
        .filter_map(|(_id, content)| {
            content
                .parse::<toml::Value>()
                .ok()
                .and_then(|v| v.get("starting_area").cloned())
        })
        .last()
        .and_then(|sa| sa.get("structures").cloned())
        .and_then(|s| s.as_array().cloned())
        .unwrap_or_default()
}

#[test]
fn map_config_starting_area_enabled() {
    let mods = test_mods();
    let cfg = MapConfig::load(&mods);
    let sa = &cfg.starting_area;
    assert!(sa.enable);
    assert_eq!(sa.radius, 8);
    assert!(sa.clear_trees);

    // Data-driven: count matches the last mod that defines [starting_area]
    let raw_structures = effective_starting_area_raw(&mods);
    assert_eq!(sa.structures.len(), raw_structures.len());
}

#[test]
fn map_config_starting_area_structures_detail() {
    let mods = test_mods();
    let cfg = MapConfig::load(&mods);
    let s = &cfg.starting_area.structures;
    let raw_structures = effective_starting_area_raw(&mods);

    assert_eq!(s.len(), raw_structures.len());

    for (i, (loaded, raw)) in s.iter().zip(raw_structures.iter()).enumerate() {
        let kind = raw
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(loaded.kind, kind, "structure[{i}] kind mismatch");

        let tile_x = raw.get("tile_x").and_then(|v| v.as_integer()).unwrap_or(0) as i32;
        let tile_y = raw.get("tile_y").and_then(|v| v.as_integer()).unwrap_or(0) as i32;
        assert_eq!(
            (loaded.tile_x, loaded.tile_y),
            (tile_x, tile_y),
            "structure[{i}] position mismatch"
        );

        let props_resource = raw
            .get("props")
            .and_then(|p| p.get("resource"))
            .and_then(|v| v.as_str());
        assert_eq!(
            loaded.props.resource.as_deref(),
            props_resource,
            "structure[{i}] props.resource mismatch"
        );

        let props_amount = raw
            .get("props")
            .and_then(|p| p.get("amount"))
            .and_then(|v| v.as_integer())
            .map(|a| a as u32);
        assert_eq!(
            loaded.props.amount,
            props_amount,
            "structure[{i}] props.amount mismatch"
        );
    }
}

#[test]
fn map_config_pathfinding_max_nodes() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.pathfinding_max_nodes, 50_000);
}

#[test]
fn map_config_builder_values() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.builder_speed, 300.0);
    assert_eq!(cfg.builder_reach, 8.0);
}

#[test]
fn map_config_decoration() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.decoration_min_count, 4);
    assert_eq!(cfg.decoration_count_variance, 5);
}

#[test]
fn map_config_inspect_range() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.inspect_range_tiles, 3.0);
    assert_eq!(cfg.builder_range_tiles, 5.0);
}

#[test]
fn map_config_mining_interval() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.player_mining_interval, 1.0);
}

#[test]
fn map_config_builder_idle_offsets() {
    let cfg = MapConfig::load(&test_mods());
    assert_eq!(cfg.builder_idle_offset_x, -24.0);
    assert_eq!(cfg.builder_idle_offset_y, -24.0);
}

// ── PlacedStructureProps ─────────────────────────────────────────────────

#[test]
fn placed_structure_props_default_none() {
    let props = PlacedStructureProps::default();
    assert!(props.resource.is_none());
    assert!(props.amount.is_none());
    assert!(props.decoration_kind.is_none());
}

#[test]
fn placed_structure_props_with_values() {
    let props = PlacedStructureProps {
        resource: Some("iron_ore".to_string()),
        amount: Some(100),
        decoration_kind: Some("tree".to_string()),
    };
    assert_eq!(props.resource.as_deref(), Some("iron_ore"));
    assert_eq!(props.amount, Some(100));
    assert_eq!(props.decoration_kind.as_deref(), Some("tree"));
}

// ── PlacedStructure ──────────────────────────────────────────────────────

#[test]
fn placed_structure_construction() {
    let s = PlacedStructure {
        kind: "hq".to_string(),
        tile_x: 10,
        tile_y: 20,
        props: PlacedStructureProps::default(),
    };
    assert_eq!(s.kind, "hq");
    assert_eq!(s.tile_x, 10);
    assert_eq!(s.tile_y, 20);
    assert!(s.props.resource.is_none());
}

#[test]
fn placed_structure_negative_coords() {
    let s = PlacedStructure {
        kind: "wall".to_string(),
        tile_x: -5,
        tile_y: -3,
        props: PlacedStructureProps {
            resource: None,
            amount: None,
            decoration_kind: None,
        },
    };
    assert_eq!(s.tile_x, -5);
    assert_eq!(s.tile_y, -3);
}

// ── StartingAreaConfig ───────────────────────────────────────────────────

#[test]
fn starting_area_config_empty() {
    let sa = StartingAreaConfig {
        enable: false,
        radius: 5,
        clear_trees: false,
        structures: vec![],
    };
    assert!(!sa.enable);
    assert_eq!(sa.radius, 5);
    assert!(!sa.clear_trees);
    assert!(sa.structures.is_empty());
}

#[test]
fn starting_area_config_with_structures() {
    let sa = StartingAreaConfig {
        enable: true,
        radius: 10,
        clear_trees: true,
        structures: vec![PlacedStructure {
            kind: "deposit".to_string(),
            tile_x: 1,
            tile_y: 2,
            props: PlacedStructureProps {
                resource: Some("iron_ore".to_string()),
                amount: Some(50),
                decoration_kind: None,
            },
        }],
    };
    assert!(sa.enable);
    assert_eq!(sa.structures.len(), 1);
    assert_eq!(sa.structures[0].props.amount, Some(50));
}

#[test]
fn starting_area_config_clone() {
    let sa = StartingAreaConfig {
        enable: true,
        radius: 8,
        clear_trees: true,
        structures: vec![],
    };
    let sa2 = sa.clone();
    assert_eq!(sa2.radius, 8);
    assert!(sa2.clear_trees);
}
