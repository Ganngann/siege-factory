use siege_factory::map::components::TileType;
use siege_factory::map::tile_grid::ChunkGrid;

fn test_dist() -> Vec<(String, u32)> {
    vec![
        ("iron_ore".to_string(), 50),
        ("copper_ore".to_string(), 35),
        ("coal".to_string(), 15),
    ]
}

#[test]
fn chunk_grid_integration() {
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, test_dist());

    let chunk = grid.ensure_chunk(0, 0);
    assert_eq!(chunk.tiles.len(), 32);
    assert_eq!(chunk.tiles[0].len(), 32);
    // Tiles at any position are accessible (infinite map)
    let tt = grid.tile_type_at(100, 200);
    assert!(tt == TileType::Ground || tt == TileType::Resource);
}

#[test]
fn chunk_grid_negative_coords() {
    let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, test_dist());
    let tt = grid.tile_type_at(-5, -10);
    assert!(tt == TileType::Ground || tt == TileType::Resource);
}
