use crate::map::components::TileType;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CHUNK_SIZE: u32 = 32;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Deposit {
    pub x: u32,
    pub y: u32,
    pub amount: u32,
    pub resource: String,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub tiles: [[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    pub deposits: Vec<Deposit>,
}

#[derive(Debug, Clone, Resource)]
pub struct ChunkGrid {
    chunks: HashMap<(i32, i32), Chunk>,
    seed: u64,
    pub deposit_min_amount: u32,
    pub deposit_max_amount: u32,
    pub deposit_spawn_chance_pct: u32,
    pub deposit_min_per_chunk: u32,
    pub deposit_max_per_chunk: u32,
    pub deposit_distribution: Vec<(String, u32)>,
}

impl ChunkGrid {
    pub fn new(
        seed: u64,
        deposit_min_amount: u32,
        deposit_max_amount: u32,
        deposit_spawn_chance_pct: u32,
        deposit_min_per_chunk: u32,
        deposit_max_per_chunk: u32,
        deposit_distribution: Vec<(String, u32)>,
    ) -> Self {
        Self {
            chunks: HashMap::new(),
            seed,
            deposit_min_amount,
            deposit_max_amount,
            deposit_spawn_chance_pct,
            deposit_min_per_chunk,
            deposit_max_per_chunk,
            deposit_distribution,
        }
    }

    // TODO: consolidate ensure_chunk and ensure_chunk_mut into a single helper
    // returning (&Chunk, &mut Chunk) or using a shared inner fn.
    pub fn ensure_chunk(&mut self, cx: i32, cy: i32) -> &Chunk {
        let seed = self.seed;
        let min_amt = self.deposit_min_amount;
        let max_amt = self.deposit_max_amount;
        let spawn_chance = self.deposit_spawn_chance_pct;
        let min_per = self.deposit_min_per_chunk;
        let max_per = self.deposit_max_per_chunk;
        let dist = self.deposit_distribution.clone();
        self.chunks.entry((cx, cy)).or_insert_with(|| {
            generate_chunk(
                seed,
                cx,
                cy,
                min_amt,
                max_amt,
                spawn_chance,
                min_per,
                max_per,
                dist,
            )
        })
    }

    pub fn ensure_chunk_mut(&mut self, cx: i32, cy: i32) -> &mut Chunk {
        let seed = self.seed;
        let min_amt = self.deposit_min_amount;
        let max_amt = self.deposit_max_amount;
        let spawn_chance = self.deposit_spawn_chance_pct;
        let min_per = self.deposit_min_per_chunk;
        let max_per = self.deposit_max_per_chunk;
        let dist = self.deposit_distribution.clone();
        self.chunks.entry((cx, cy)).or_insert_with(|| {
            generate_chunk(
                seed,
                cx,
                cy,
                min_amt,
                max_amt,
                spawn_chance,
                min_per,
                max_per,
                dist,
            )
        })
    }

    pub fn tile_type_at(&mut self, x: i32, y: i32) -> TileType {
        let cx = x.div_euclid(CHUNK_SIZE as i32);
        let cy = y.div_euclid(CHUNK_SIZE as i32);
        let tx = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let ty = y.rem_euclid(CHUNK_SIZE as i32) as usize;
        self.ensure_chunk(cx, cy).tiles[ty][tx]
    }

    pub fn chunk_containing(&self, x: i32, y: i32) -> (i32, i32) {
        (
            x.div_euclid(CHUNK_SIZE as i32),
            y.div_euclid(CHUNK_SIZE as i32),
        )
    }

    pub fn generated_chunks(&self) -> impl Iterator<Item = &(i32, i32)> {
        self.chunks.keys()
    }

    pub fn chunk_exists(&self, cx: i32, cy: i32) -> bool {
        self.chunks.contains_key(&(cx, cy))
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub fn get_chunk(&self, cx: i32, cy: i32) -> Option<&Chunk> {
        self.chunks.get(&(cx, cy))
    }

    pub fn generated_chunks_with_data(&self) -> impl Iterator<Item = (&(i32, i32), &Chunk)> {
        self.chunks.iter()
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn set_deposit_amount(&mut self, cx: i32, cy: i32, dx: u32, dy: u32, amount: u32) {
        if let Some(chunk) = self.chunks.get_mut(&(cx, cy)) {
            for d in &mut chunk.deposits {
                if d.x == dx && d.y == dy {
                    d.amount = amount;
                    return;
                }
            }
        }
    }

    pub fn set_deposit_resource(&mut self, cx: i32, cy: i32, dx: u32, dy: u32, resource: &str) {
        if let Some(chunk) = self.chunks.get_mut(&(cx, cy)) {
            for d in &mut chunk.deposits {
                if d.x == dx && d.y == dy {
                    d.resource = resource.to_string();
                    return;
                }
            }
        }
    }
}

fn generate_chunk(
    seed: u64,
    cx: i32,
    cy: i32,
    deposit_min_amount: u32,
    deposit_max_amount: u32,
    deposit_spawn_chance_pct: u32,
    deposit_min_per_chunk: u32,
    deposit_max_per_chunk: u32,
    deposit_distribution: Vec<(String, u32)>,
) -> Chunk {
    use super::rng::{SimpleRng, chunk_hash};

    let mut tiles = [[TileType::Ground; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

    let h = chunk_hash(seed, cx, cy);

    let mut rng = SimpleRng::new(h);

    for ty in 0..CHUNK_SIZE as usize {
        for tx in 0..CHUNK_SIZE as usize {
            tiles[ty][tx] = TileType::Ground;
        }
    }

    let total_weight: u32 = deposit_distribution.iter().map(|(_, w)| w).sum();
    let has_deposits = rng.next() % 100 < deposit_spawn_chance_pct;
    let mut deposits = Vec::new();

    if has_deposits && total_weight > 0 {
        let count_range = deposit_max_per_chunk - deposit_min_per_chunk;
        let count = deposit_min_per_chunk + (rng.next() % (count_range + 1));
        for _ in 0..count {
            let dx = rng.next() % CHUNK_SIZE;
            let dy = rng.next() % CHUNK_SIZE;
            let amount = deposit_min_amount
                + (rng.next() % (deposit_max_amount - deposit_min_amount + 1));
            let pick = rng.next() % total_weight;
            let mut cumulative = 0u32;
            let resource = deposit_distribution
                .iter()
                .find(|(_, w)| {
                    cumulative += w;
                    pick < cumulative
                })
                .map(|(r, _)| r.clone())
                .unwrap_or_else(|| "iron_ore".to_string());
            tiles[dy as usize][dx as usize] = TileType::Resource;
            deposits.push(Deposit { x: dx, y: dy, amount, resource });
        }
    }

    Chunk { tiles, deposits }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_chunk() {
        let dist = vec![
            ("iron_ore".to_string(), 50),
            ("copper_ore".to_string(), 35),
            ("coal".to_string(), 15),
        ];
        let mut a = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist.clone());
        let mut b = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist.clone());
        let chunk_a = a.ensure_chunk(0, 0).clone();
        let chunk_b = b.ensure_chunk(0, 0).clone();
        assert_eq!(chunk_a.tiles, chunk_b.tiles);
        assert_eq!(chunk_a.deposits, chunk_b.deposits);
    }

    #[test]
    fn deterministic_generation() {
        let dist = vec![
            ("iron_ore".to_string(), 50),
            ("copper_ore".to_string(), 35),
            ("coal".to_string(), 15),
        ];
        let mut g1 = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist.clone());
        let mut g2 = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist.clone());
        assert_eq!(g1.ensure_chunk(0, 0).tiles, g2.ensure_chunk(0, 0).tiles);
        assert_eq!(
            g1.ensure_chunk(0, 0).deposits,
            g2.ensure_chunk(0, 0).deposits
        );
    }

    #[test]
    fn different_chunks_are_independent() {
        let dist = vec![
            ("iron_ore".to_string(), 50),
            ("copper_ore".to_string(), 35),
            ("coal".to_string(), 15),
        ];
        let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist.clone());
        let _ = grid.ensure_chunk(1, 0).clone();
        let c0 = grid.ensure_chunk(0, 0).clone();

        let mut grid2 = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist.clone());
        let c0_fresh = grid2.ensure_chunk(0, 0).clone();

        assert_eq!(
            c0.tiles, c0_fresh.tiles,
            "tiles must be independent of generation order"
        );
        assert_eq!(
            c0.deposits, c0_fresh.deposits,
            "deposits must be independent of generation order"
        );
    }

    #[test]
    fn chunk_containing_rounds_correctly() {
        let dist = vec![
            ("iron_ore".to_string(), 50),
            ("copper_ore".to_string(), 35),
            ("coal".to_string(), 15),
        ];
        let grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
        assert_eq!(grid.chunk_containing(0, 0), (0, 0));
        assert_eq!(grid.chunk_containing(31, 31), (0, 0));
        assert_eq!(grid.chunk_containing(32, 0), (1, 0));
        assert_eq!(grid.chunk_containing(-1, 0), (-1, 0));
    }

    #[test]
    fn tile_type_at_auto_generates_chunk() {
        let dist = vec![
            ("iron_ore".to_string(), 50),
            ("copper_ore".to_string(), 35),
            ("coal".to_string(), 15),
        ];
        let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
        let tt = grid.tile_type_at(100, 200);
        assert!(tt == TileType::Ground || tt == TileType::Resource);
    }

    #[test]
    fn deposits_are_valid() {
        let dist = vec![
            ("iron_ore".to_string(), 50),
            ("copper_ore".to_string(), 35),
            ("coal".to_string(), 15),
        ];
        let mut grid = ChunkGrid::new(42, 50, 150, 35, 2, 5, dist);
        let chunk = grid.ensure_chunk(0, 0);
        for d in &chunk.deposits {
            assert!(d.x < CHUNK_SIZE);
            assert!(d.y < CHUNK_SIZE);
            assert!(d.amount >= 50);
            assert!(d.amount <= 150);
        }
    }
}
