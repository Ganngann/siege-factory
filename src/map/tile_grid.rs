use crate::map::components::Tile;

#[derive(Debug, Clone)]
pub struct TileGrid {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl TileGrid {
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![Tile::default(); (width * height) as usize];
        Self { tiles, width, height }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn tile_at(&self, x: u32, y: u32) -> Result<&Tile, TileError> {
        if x >= self.width || y >= self.height {
            return Err(TileError::OutOfBounds { x, y, width: self.width, height: self.height });
        }
        Ok(&self.tiles[(y * self.width + x) as usize])
    }

    pub fn tile_at_mut(&mut self, x: u32, y: u32) -> Result<&mut Tile, TileError> {
        if x >= self.width || y >= self.height {
            return Err(TileError::OutOfBounds { x, y, width: self.width, height: self.height });
        }
        Ok(&mut self.tiles[(y * self.width + x) as usize])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileError {
    OutOfBounds { x: u32, y: u32, width: u32, height: u32 },
}

impl std::fmt::Display for TileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileError::OutOfBounds { x, y, width, height } => {
                write!(f, "position ({x}, {y}) is out of bounds for grid {width}x{height}")
            }
        }
    }
}

impl std::error::Error for TileError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_grid_with_correct_dimensions() {
        let grid = TileGrid::new(20, 15);
        assert_eq!(grid.width(), 20);
        assert_eq!(grid.height(), 15);
    }

    #[test]
    fn tile_at_returns_tile_for_valid_coordinates() {
        let grid = TileGrid::new(20, 15);
        let tile = grid.tile_at(5, 5);
        assert!(tile.is_ok());
        assert_eq!(*tile.unwrap(), Tile::default());
    }

    #[test]
    fn tile_at_returns_error_for_out_of_bounds_x() {
        let grid = TileGrid::new(20, 15);
        let result = grid.tile_at(20, 5);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            TileError::OutOfBounds { x: 20, y: 5, width: 20, height: 15 }
        );
    }

    #[test]
    fn tile_at_returns_error_for_out_of_bounds_y() {
        let grid = TileGrid::new(20, 15);
        let result = grid.tile_at(5, 15);
        assert!(result.is_err());
    }

    #[test]
    fn tile_at_mut_allows_modification() {
        let mut grid = TileGrid::new(20, 15);
        let tile = grid.tile_at_mut(10, 10).unwrap();
        tile.occupied = true;
        assert!(grid.tile_at(10, 10).unwrap().occupied);
    }

    #[test]
    fn default_tile_is_ground_and_unoccupied() {
        let grid = TileGrid::new(1, 1);
        let tile = grid.tile_at(0, 0).unwrap();
        assert_eq!(tile.tile_type, crate::map::components::TileType::Ground);
        assert!(!tile.occupied);
    }

    proptest::proptest! {
        #[test]
        fn grid_always_returns_tile_or_out_of_bounds(
            grid_w in 1..100u32,
            grid_h in 1..100u32,
            x in 0..200u32,
            y in 0..200u32,
        ) {
            let grid = TileGrid::new(grid_w, grid_h);
            let result = grid.tile_at(x, y);
            if x < grid_w && y < grid_h {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }
}
