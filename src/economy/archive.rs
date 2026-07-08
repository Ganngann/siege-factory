use bevy::prelude::*;

use crate::core::toast::ToastQueue;
use crate::economy::components::{Archive, DiscoveredRecipes};
use crate::economy::discovery::GlobalArchive;
use crate::economy::resource::{Inventory, ResourceRegistry};

pub fn archive_delivery_check(
    mut commands: Commands,
    archive_query: Query<(Entity, &Inventory), With<Archive>>,
    discovered_query: Query<&DiscoveredRecipes>,
    mut global_archive: ResMut<GlobalArchive>,
    resource_registry: Res<ResourceRegistry>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let mut to_unlock: Vec<String> = Vec::new();
    let mut to_clear: Vec<Entity> = Vec::new();

    for (entity, inventory) in &archive_query {
        for (resource_id, _amount) in inventory.iter_occupied() {
            if global_archive.is_unlocked(&resource_id.0) {
                continue;
            }

            let is_pending = discovered_query
                .iter()
                .any(|d| d.0.iter().any(|id| id == &resource_id.0));
            if !is_pending {
                continue;
            }

            to_unlock.push(resource_id.0.clone());
            to_clear.push(entity);
        }
    }

    for rid in &to_unlock {
        global_archive.unlocked_recipes.insert(rid.clone());
        let name = resource_registry
            .get_opt(rid)
            .map(|r| r.name.as_str())
            .unwrap_or(rid);
        toast_queue.0.push(format!("Archived: {}!", name));
    }

    for entity in to_clear {
        commands.entity(entity).insert(Inventory::with_capacity(1));
    }
}
