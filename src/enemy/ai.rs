use bevy::prelude::*;
use crate::enemy::components::Enemy;
use crate::enemy::registry::EnemyRegistry;
use crate::economy::components::{HQ, Building};
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use std::collections::VecDeque;

fn bfs(
    start: (u32, u32),
    goal: (u32, u32),
    blocked: &[bool],
    grid_w: u32,
    grid_h: u32,
) -> Option<Vec<(u32, u32)>> {
    let size = (grid_w * grid_h) as usize;
    let mut queue = VecDeque::new();
    let mut visited = vec![false; size];
    let mut parent = vec![None; size];

    let start_idx = start.1 as usize * grid_w as usize + start.0 as usize;
    if blocked[start_idx] {
        return None;
    }

    queue.push_back(start);
    visited[start_idx] = true;

    while let Some((cx, cy)) = queue.pop_front() {
        if cx == goal.0 && cy == goal.1 {
            let mut path = Vec::new();
            let mut cur = (goal.0, goal.1);
            while cur != start {
                path.push(cur);
                let idx = cur.1 as usize * grid_w as usize + cur.0 as usize;
                cur = parent[idx].unwrap();
            }
            path.reverse();
            return Some(path);
        }

        for (dx, dy) in [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)] {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && nx < grid_w as i32 && ny >= 0 && ny < grid_h as i32 {
                let nx = nx as u32;
                let ny = ny as u32;
                let idx = ny as usize * grid_w as usize + nx as usize;
                if !visited[idx] && !blocked[idx] {
                    visited[idx] = true;
                    parent[idx] = Some((cx, cy));
                    queue.push_back((nx, ny));
                }
            }
        }
    }
    None
}

pub fn move_enemies(
    mut set: ParamSet<(
        Query<(Entity, &mut Transform, &mut TilePosition), With<Enemy>>,
        Query<&TilePosition, With<HQ>>,
        Query<&TilePosition, (With<Building>, Without<HQ>)>,
    )>,
    time: Res<Time>,
    enemies_registry: Res<EnemyRegistry>,
    cfg: Res<MapConfig>,
) {
    let grid_w = cfg.width;
    let grid_h = cfg.height;
    let tile_size = cfg.tile_size;

    let enemy_speed = enemies_registry.get("runner")
        .map(|d| d.speed)
        .unwrap_or(60.0);

    let hq_pos = match set.p1().get_single() {
        Ok(p) => *p,
        Err(_) => return,
    };
    let goal = (hq_pos.x, hq_pos.y);
    let mut blocked = vec![false; (grid_w * grid_h) as usize];
    for &pos in set.p2().iter() {
        let idx = pos.y as usize * grid_w as usize + pos.x as usize;
        blocked[idx] = true;
    }

    for (_entity, mut transform, mut pos) in set.p0().iter_mut() {
        let start = (pos.x, pos.y);
        if start == goal {
            continue;
        }

        let path = bfs(start, goal, &blocked, grid_w, grid_h);
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
            let step = enemy_speed * time.delta_seconds();
            let ratio = (step / dist).min(1.0);
            transform.translation.x += dx * ratio;
            transform.translation.y += dy * ratio;
        }
    }
}
