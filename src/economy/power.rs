use bevy::prelude::*;
use std::collections::HashSet;

use crate::economy::components::{PowerConsumer, PowerPole, PowerProducer, UnbuiltBuilding};

#[derive(Resource, Default)]
pub struct PowerGrid {
    pub dirty: bool,
}

pub fn detect_power_changes(
    mut grid: ResMut<PowerGrid>,
    added: Query<Entity, Or<(Added<PowerConsumer>, Added<PowerProducer>, Added<PowerPole>)>>,
) {
    if !added.is_empty() {
        grid.dirty = true;
    }
}

fn is_in_range(pos: Vec3, poles: &[(Entity, Vec3, f32)]) -> bool {
    poles.iter().any(|(_, pp, range)| pp.distance(pos) <= *range)
}

pub fn rebuild_power_grid(
    mut grid: ResMut<PowerGrid>,
    producers: Query<(Entity, &PowerProducer, &Transform), Without<UnbuiltBuilding>>,
    poles: Query<(Entity, &PowerPole, &Transform), Without<UnbuiltBuilding>>,
    mut consumers: Query<(Entity, &mut PowerConsumer, &Transform), Without<UnbuiltBuilding>>,
) {
    if !grid.dirty {
        return;
    }
    grid.dirty = false;

    let pole_data: Vec<(Entity, Vec3, f32)> = poles
        .iter()
        .map(|(e, p, tf)| (e, tf.translation, p.range))
        .collect();

    let mut connected_producers: HashSet<Entity> = HashSet::new();
    let mut consumer_map: HashSet<Entity> = HashSet::new();

    for (entity, _producer, tf) in producers.iter() {
        if is_in_range(tf.translation, &pole_data) || pole_data.is_empty() {
            connected_producers.insert(entity);
        }
    }

    for (entity, _consumer, tf) in consumers.iter() {
        if is_in_range(tf.translation, &pole_data) || pole_data.is_empty() {
            consumer_map.insert(entity);
        }
    }

    let total_production: f32 = producers
        .iter()
        .filter(|(e, _, _)| connected_producers.contains(e))
        .map(|(_, p, _)| p.output)
        .sum();

    let total_consumption: f32 = consumers
        .iter()
        .filter(|(e, _, _)| consumer_map.contains(e))
        .map(|(_, c, _)| c.draw)
        .sum();

    let power_ok = total_production >= total_consumption && total_production > 0.0;

    for (entity, mut consumer, _) in consumers.iter_mut() {
        consumer.satisfied = consumer_map.contains(&entity) && power_ok;
    }
}
