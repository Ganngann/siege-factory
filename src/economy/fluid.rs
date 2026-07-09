use bevy::prelude::*;

use crate::economy::components::{Active, Direction, PowerConsumer, UnbuiltBuilding};
use crate::economy::game_components::Pump;
use crate::economy::resource::ResourceId;
use crate::economy::spatial::SpatialRegistry;
use crate::map::components::TilePosition;

// ── FluidTank ──

#[derive(Component, Clone)]
pub struct FluidTank {
    pub fluids: Vec<(ResourceId, f32)>,
    pub capacity: f32,
    pub max_per_fluid: f32,
}

impl FluidTank {
    pub fn new(capacity: f32) -> Self {
        Self {
            fluids: Vec::new(),
            capacity,
            max_per_fluid: 0.0,
        }
    }

    pub fn with_max_per_fluid(capacity: f32, max_per_fluid: f32) -> Self {
        Self {
            fluids: Vec::new(),
            capacity,
            max_per_fluid,
        }
    }

    pub fn get(&self, resource: &ResourceId) -> f32 {
        self.fluids
            .iter()
            .find(|(r, _)| r == resource)
            .map(|(_, a)| *a)
            .unwrap_or(0.0)
    }

    pub fn add(&mut self, resource: &ResourceId, amount: f32) -> f32 {
        let effective = if self.capacity > 0.0 {
            let room = self.capacity - self.total();
            if room <= 0.0 {
                return 0.0;
            }
            amount.min(room)
        } else {
            amount
        };

        if self.max_per_fluid > 0.0 {
            let current = self.get(resource);
            let limit_room = self.max_per_fluid - current;
            if limit_room <= 0.0 {
                return 0.0;
            }
            let effective = effective.min(limit_room);
            if effective <= 0.0 {
                return 0.0;
            }
            if let Some((_, a)) = self.fluids.iter_mut().find(|(r, _)| r == resource) {
                *a += effective;
            } else {
                self.fluids.push((resource.clone(), effective));
            }
            return effective;
        }

        if let Some((_, a)) = self.fluids.iter_mut().find(|(r, _)| r == resource) {
            *a += effective;
        } else {
            self.fluids.push((resource.clone(), effective));
        }
        effective
    }

    pub fn remove(&mut self, resource: &ResourceId, amount: f32) -> f32 {
        for (r, a) in self.fluids.iter_mut() {
            if r == resource {
                let removed = a.min(amount);
                *a -= removed;
                if *a <= 0.0 {
                    self.fluids.retain(|(res, _)| res != resource);
                }
                return removed;
            }
        }
        0.0
    }

    pub fn total(&self) -> f32 {
        self.fluids.iter().map(|(_, a)| a).sum()
    }

    pub fn is_full(&self) -> bool {
        self.capacity > 0.0 && self.total() >= self.capacity
    }
}

// ── Pipe component ──

#[derive(Component)]
pub struct FluidPipe {
    pub transfer_rate: f32,
    pub direction: Direction,
}

// ── Pipe transfer system ──
// Each tick, a pipe pushes fluid from the tile behind it (opposite direction)
// to the tile in front of it.

pub fn fluid_pipe_transfer(
    time: Res<Time<Fixed>>,
    spatial: Res<SpatialRegistry>,
    pipes: Query<(
        &TilePosition,
        &FluidPipe,
    )>,
    mut tanks: Query<&mut FluidTank>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (pos, pipe) in pipes.iter() {
        let dir = &pipe.direction;
        let (fdx, fdy) = dir.offset();
        let (bdx, bdy) = (-fdx, -fdy);

        let src_x = pos.x + bdx;
        let src_y = pos.y + bdy;
        let dst_x = pos.x + fdx;
        let dst_y = pos.y + fdy;

        let src_entity = match spatial.at(src_x, src_y) {
            Some(e) => e,
            None => continue,
        };
        let dst_entity = match spatial.at(dst_x, dst_y) {
            Some(e) => e,
            None => continue,
        };

        // Phase 1: read source data, then release borrow
        let (transfer_resource, available) = {
            let Ok(src_tank) = tanks.get(src_entity) else { continue };
            let mut best: Option<(ResourceId, f32)> = None;
            for (res, amt) in &src_tank.fluids {
                if *amt > 0.0 {
                    if let Some((_, best_amt)) = &best {
                        if amt > best_amt {
                            best = Some((res.clone(), *amt));
                        }
                    } else {
                        best = Some((res.clone(), *amt));
                    }
                }
            }
            match best {
                Some(r) => r,
                None => continue,
            }
        };

        let max_transfer = pipe.transfer_rate * dt;
        let to_send = max_transfer.min(available);
        if to_send <= 0.0 {
            continue;
        }

        // Phase 2: try to add to destination
        let accepted = {
            let Ok(mut dst_tank) = tanks.get_mut(dst_entity) else { continue };
            if dst_tank.is_full() {
                continue;
            }
            dst_tank.add(&transfer_resource, to_send)
        };

        // Phase 3: remove from source
        if accepted > 0.0 {
            if let Ok(mut src_mut) = tanks.get_mut(src_entity) {
                src_mut.remove(&transfer_resource, accepted);
            }
        }
    }
}

// ── Water pump system ──

pub fn water_pump_tick(
    time: Res<Time<Fixed>>,
    mut pump_query: Query<
        (&mut FluidTank, &Active, Option<&PowerConsumer>),
        (With<Pump>, Without<UnbuiltBuilding>),
    >,
) {
    for (mut tank, active, power) in pump_query.iter_mut() {
        if !active.0 {
            continue;
        }
        if let Some(pc) = power {
            if !pc.satisfied {
                continue;
            }
        }
        if tank.is_full() {
            continue;
        }
        let water = ResourceId::new("water");
        tank.add(&water, 10.0 * time.delta_secs());
    }
}
