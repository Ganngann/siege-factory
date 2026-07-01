use siege_factory::map::tile_grid::TileGrid;

#[test]
fn grid_creation_integration() {
    let grid = TileGrid::new(30, 20);
    assert_eq!(grid.width(), 30);
    assert_eq!(grid.height(), 20);
    assert!(grid.tile_at(0, 0).is_ok());
    assert!(grid.tile_at(29, 19).is_ok());
    assert!(grid.tile_at(30, 20).is_err());
}
