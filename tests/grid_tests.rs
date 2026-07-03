use siege_factory::map::tile_grid::ChunkGrid;
use siege_factory::map::components::TileType;

#[test]
fn chunk_grid_integration() {
    let mut grid = ChunkGrid::new(42);
    let chunk = grid.ensure_chunk(0, 0);
    assert_eq!(chunk.tiles.len(), 32);
    assert_eq!(chunk.tiles[0].len(), 32);
    // Tiles at any position are accessible (infinite map)
    let tt = grid.tile_type_at(100, 200);
    assert!(tt == TileType::Ground || tt == TileType::Resource);
}

#[test]
fn chunk_grid_negative_coords() {
    let mut grid = ChunkGrid::new(42);
    let tt = grid.tile_type_at(-5, -10);
    assert!(tt == TileType::Ground || tt == TileType::Resource);
}
