use bevy::prelude::*;

use crate::economy::components::{Active, Compactor, PowerConsumer};
use crate::economy::resource::ResourceId;

pub fn compactor_tick(
    time: Res<Time<Fixed>>,
    mut compactor_query: Query<(
        &mut Compactor,
        &mut crate::economy::resource::Inventory,
        &Active,
        Option<&PowerConsumer>,
    )>,
) {
    for (mut compactor, mut inventory, active, power) in compactor_query.iter_mut() {
        if !active.0 {
            continue;
        }
        if let Some(pc) = power
            && !pc.satisfied {
                continue;
            }

        compactor.timer += time.delta_secs();
        if compactor.timer < compactor.interval {
            continue;
        }
        compactor.timer -= compactor.interval;

        // Find the most-abundant resource with count >= ratio
        let target_resource = {
            let mut best: Option<(ResourceId, u32)> = None;
            for slot in inventory.slots.iter() {
                if let Some((res, amt)) = slot
                    && *amt >= compactor.ratio {
                        if let Some((_, current_best)) = &best {
                            if amt > current_best {
                                best = Some((res.clone(), *amt));
                            }
                        } else {
                            best = Some((res.clone(), *amt));
                        }
                    }
            }
            best
        };

        let Some((resource, _)) = target_resource else {
            continue;
        };

        // Check room for the compressed item
        let compressed_id = ResourceId(format!("{}_compressed", resource.0));
        if inventory.capacity > 0 {
            let new_total = inventory.total() - compactor.ratio + 1;
            if new_total > inventory.capacity {
                continue;
            }
        }

        // Consume ratio items, produce 1 compressed
        inventory.remove(&resource, compactor.ratio);
        inventory.add(&compressed_id, 1);
    }
}
