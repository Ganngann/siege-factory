use bevy::prelude::*;

#[derive(Component, Default)]
pub struct ProductionCounter(pub u32);

#[derive(Component, Default)]
pub struct DiscoveredRecipes(pub Vec<String>);

#[derive(Component)]
pub struct Archive;
