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
