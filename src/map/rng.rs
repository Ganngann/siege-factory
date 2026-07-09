use std::hash::{Hash, Hasher};

pub struct SimpleRng(pub u64);

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    // SUGGEST: envisager d'implémenter std::iter::Iterator ou de renommer (clippy::should_implement_trait)
    pub fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

pub fn chunk_hash(seed: u64, cx: i32, cy: i32) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (seed, cx, cy).hash(&mut hasher);
    hasher.finish()
}
