use crate::economy::belt::{BeltSlots, compute_slot_positions};
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{Active, Building, OccupiedTiles, Sorter, Splitter};
use crate::economy::game_components::{Level, Player};
use crate::economy::resource::{Cost, Inventory, ResourceId};
use crate::events::BeltDragCompleted;
use crate::map::components::TilePosition;
use crate::map::config::MapConfig;
use bevy::prelude::*;

const BUILDING_SPLITTER: &str = "splitter";
const BUILDING_SORTER: &str = "sorter";

fn can_afford(player_inv: &Inventory, cost: &[Cost]) -> bool {
    cost.iter().all(|c| player_inv.get(&c.resource) >= c.amount)
}

fn deduct_cost(player_inv: &mut Inventory, cost: &[Cost]) {
    for c in cost {
        player_inv.remove(&c.resource, c.amount);
    }
}

/// Observer for `BeltDragCompleted`. Handles cost deduction, existing belt
/// direction updates, and spawning new belt/splitter/sorter entities.
pub fn on_belt_drag_completed(
    on: On<BeltDragCompleted>,
    mut commands: Commands,
    mut belt_write: Query<(&TilePosition, &mut BeltSlots)>,
    mut player_query: Query<&mut Inventory, With<Player>>,
    registry: Res<BuildingRegistry>,
    cfg: Res<MapConfig>,
    mut toast_queue: ResMut<crate::core::toast::ToastQueue>,
) {
    let ev = on.event();
    let Some(def) = registry.get(&ev.kind) else {
        return;
    };
    let tile_size = cfg.tile_size;

    if !ev.new_tiles.is_empty() {
        let mut player_inv = match player_query.single_mut() {
            Ok(inv) => inv,
            Err(_) => return,
        };
        let scaled_cost: Vec<Cost> = def
            .cost
            .iter()
            .map(|c| Cost {
                resource: c.resource.clone(),
                amount: c.amount * ev.new_tiles.len() as u32,
            })
            .collect();
        if !can_afford(&player_inv, &scaled_cost) {
            toast_queue.0.push("Not enough resources".to_string());
            return;
        }
        deduct_cost(&mut player_inv, &scaled_cost);
    }

    if ev.new_tiles.is_empty() && ev.existing.is_empty() {
        return;
    }

    if def.belt.is_some() {
        let num_slots = def.belt.as_ref().map_or(2, |b| b.slots);
        let speed = def.belt.as_ref().map_or(2.0, |b| b.speed);

        for &(bx, by, dir) in &ev.existing {
            if let Some((_, mut bs)) = belt_write
                .iter_mut()
                .find(|(pos, _)| pos.x == bx && pos.y == by)
            {
                if bs.direction != dir {
                    bs.direction = dir;
                    bs.slot_positions = compute_slot_positions(bx, by, dir, num_slots, tile_size);
                }
            }
        }

        for &(bx, by, dir) in &ev.new_tiles {
            let cx = bx as f32 * tile_size;
            let cy = by as f32 * tile_size;
            let slot_positions = compute_slot_positions(bx, by, dir, num_slots, tile_size);
            let items: Vec<Option<crate::economy::belt::ItemOnBelt>> =
                vec![None; num_slots as usize];
            let slot_sprites: Vec<Option<Entity>> = vec![None; num_slots as usize];

            let belt_components = (
                Building {
                    kind: def.id.clone(),
                    name: def.name.clone(),
                },
                Inventory::new(),
                OccupiedTiles(vec![(bx, by)]),
                TilePosition { x: bx, y: by },
                BeltSlots {
                    direction: dir,
                    items,
                    slot_sprites,
                    slot_positions,
                    speed,
                },
                Transform::from_xyz(cx, cy, 2.0)
                    .with_rotation(Quat::from_rotation_z(dir.to_angle())),
            );

            if def.id == BUILDING_SPLITTER {
                commands.spawn((
                    belt_components,
                    Splitter {
                        counter: 0,
                        outputs: 2,
                        input_direction: None,
                    },
                    def.belt_variant,
                    Active(true),
                    Level(def.level),
                ));
            } else if def.id == BUILDING_SORTER {
                let filter = def
                    .default_filter
                    .clone()
                    .unwrap_or_else(|| "iron_ore".to_string());
                commands.spawn((
                    belt_components,
                    Sorter {
                        filter: ResourceId(filter),
                        inverted: false,
                    },
                    def.belt_variant,
                    Active(true),
                    Level(def.level),
                ));
            } else {
                commands.spawn((belt_components, def.belt_variant, Active(true), Level(def.level)));
            }
        }
    } else if def.drag_placement {
        for &(bx, by, _dir) in &ev.new_tiles {
            let cx = bx as f32 * tile_size;
            let cy = by as f32 * tile_size;
            commands.spawn((
                Building {
                    kind: def.id.clone(),
                    name: def.name.clone(),
                },
                Inventory::new(),
                OccupiedTiles(vec![(bx, by)]),
                TilePosition { x: bx, y: by },
                Transform::from_xyz(cx, cy, 2.0),
                Active(true),
                Level(def.level),
            ));
        }
    }
}
