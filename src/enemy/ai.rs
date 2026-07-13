#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
use crate::core::utils::{move_toward, tile_to_world};
use crate::economy::components::Player;
use crate::economy::spatial::SpatialRegistry;
use crate::enemy::components::Enemy;
use crate::enemy::registry::EnemyRegistry;
use crate::enemy::wave_config::WaveConfig;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn bfs(
    start: (i32, i32),
    goal: (i32, i32),
    blocked: &HashSet<(i32, i32)>,
    max_nodes: usize,
) -> Option<Vec<(i32, i32)>> {
    let mut visited = HashSet::new();
    let mut parent: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut queue = VecDeque::new();

    if blocked.contains(&start) {
        return None;
    }

    visited.insert(start);
    queue.push_back(start);

    while let Some(pos) = queue.pop_front() {
        if visited.len() > max_nodes {
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
    // SUGGEST: type EnemySet = ParamSet<(Query<(Entity, &Enemy, &mut Transform, &mut TilePosition)>, Query<&TilePosition, With<Player>>)> (clippy::type_complexity)
    mut set: ParamSet<(
        Query<(Entity, &Enemy, &mut Transform, &mut TilePosition)>,
        Query<&TilePosition, With<Player>>,
    )>,
    time: Res<Time>,
    spatial: Res<SpatialRegistry>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<MapConfig>,
    wave_cfg: Res<WaveConfig>,
) {
    let tile_size = cfg.tile_size;

    let player_pos = match set.p1().single() {
        Ok(p) => *p,
        Err(_) => return,
    };
    let goal = (player_pos.x, player_pos.y);

    // Build blocked set from spatial registry
    let blocked: HashSet<(i32, i32)> = spatial.occupied_tiles().copied().collect();

    for (_entity, enemy, mut transform, mut pos) in set.p0().iter_mut() {
        let enemy_speed = enemies_registry
            .get(&enemy.kind)
            .map(|d| d.speed)
            .unwrap_or(60.0);

        let start = (pos.x, pos.y);
        if start == goal {
            continue;
        }

        let path = bfs(start, goal, &blocked, cfg.pathfinding_max_nodes);
        let target = match path {
            Some(ref p) if !p.is_empty() => (p[0].0, p[0].1),
            _ => continue,
        };

        let target_pos = tile_to_world(target.0, target.1, tile_size);
        let target_wx = target_pos.x;
        let target_wy = target_pos.y;
        let dx = target_wx - transform.translation.x;
        let dy = target_wy - transform.translation.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < wave_cfg.enemy_arrival_threshold {
            pos.x = target.0;
            pos.y = target.1;
            transform.translation.x = target_wx;
            transform.translation.y = target_wy;
        } else {
            let z = transform.translation.z;
            move_toward(
                &mut transform.translation,
                Vec3::new(target_wx, target_wy, z),
                enemy_speed,
                time.delta_secs(),
            );
        }
    }
}


