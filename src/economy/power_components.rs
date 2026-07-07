use bevy::prelude::*;

#[derive(Component)]
pub struct PowerConsumer {
    pub draw: f32,
    pub satisfied: bool,
}

#[derive(Component)]
pub struct PowerProducer {
    pub output: f32,
}

#[derive(Component)]
pub struct PowerPole {
    pub range: f32,
}

#[derive(Component)]
pub struct BurnerGenerator {
    pub fuel_burn_timer: f32,
    pub fuel_burn_interval: f32,
    pub base_output: f32,
}
