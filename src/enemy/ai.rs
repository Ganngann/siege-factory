use crate::economy::components::{HQ, OccupiedTiles};
use crate::economy::spatial::SpatialRegistry;
use crate::enemy::components::Enemy;
use crate::enemy::registry::EnemyRegistry;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

fn bfs(
    start: (i32, i32),
    goal: (i32, i32),
    blocked: &HashSet<(i32, i32)>,
) -> Option<Vec<(i32, i32)>> {
    const MAX_NODES: usize = 50_000;
    let mut visited = HashSet::new();
    let mut parent: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut queue = VecDeque::new();

    if blocked.contains(&start) {
        return None;
    }

    visited.insert(start);
    queue.push_back(start);

    while let Some(pos) = queue.pop_front() {
        if visited.len() > MAX_NODES {
            return None;
        }
        if pos == goal {
            let mut path = Vec::new();
            let mut cur = goal;
            while cur != start {
                path.push(cur);
                cur = parent[&cur];
            }
            path.reverse();
            return Some(path);
        }

        for (dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let next = (pos.0 + dx, pos.1 + dy);
            if !blocked.contains(&next) && visited.insert(next) {
                parent.insert(next, pos);
                queue.push_back(next);
            }
        }
    }
    None
}

pub fn move_enemies(
    mut set: ParamSet<(
        Query<(Entity, &Enemy, &mut Transform, &mut TilePosition)>,
        Query<&TilePosition, With<HQ>>,
    )>,
    time: Res<Time>,
    spatial: Res<SpatialRegistry>,
    hq_tiles_query: Query<&OccupiedTiles, With<HQ>>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<MapConfig>,
) {
    let tile_size = cfg.tile_size;

    let hq_pos = match set.p1().single() {
        Ok(p) => *p,
        Err(_) => return,
    };
    let goal = (hq_pos.x, hq_pos.y);

    // Build blocked set from spatial registry, excluding HQ tiles
    let hq_tiles: HashSet<(i32, i32)> = hq_tiles_query
        .iter()
        .flat_map(|t| t.0.iter().copied())
        .collect();
    let mut blocked: HashSet<(i32, i32)> = spatial.occupied_tiles().copied().collect();
    for tile in &hq_tiles {
        blocked.remove(tile);
    }

    for (_entity, enemy, mut transform, mut pos) in set.p0().iter_mut() {
        let enemy_speed = enemies_registry
            .get(&enemy.kind)
            .map(|d| d.speed)
            .unwrap_or(60.0);

        let start = (pos.x, pos.y);
        if start == goal {
            continue;
        }

        let path = bfs(start, goal, &blocked);
        let target = match path {
            Some(ref p) if !p.is_empty() => (p[0].0, p[0].1),
            _ => continue,
        };

        let target_wx = target.0 as f32 * tile_size + tile_size / 2.0;
        let target_wy = target.1 as f32 * tile_size + tile_size / 2.0;
        let dx = target_wx - transform.translation.x;
        let dy = target_wy - transform.translation.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 2.0 {
            pos.x = target.0;
            pos.y = target.1;
            transform.translation.x = target_wx;
            transform.translation.y = target_wy;
        } else {
            let step = enemy_speed * time.delta_secs();
            let ratio = (step / dist).min(1.0);
            transform.translation.x += dx * ratio;
            transform.translation.y += dy * ratio;
        }
    }
}
