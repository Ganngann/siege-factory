use bevy::prelude::*;

use crate::economy::components::{Active, Building};
use crate::economy::game_components::CurrentTier;
use crate::economy::resource::Inventory;

/// Résout les clés de données comme "building.name" ou "health.current"
/// en valeurs String à partir des composants ECS de l'entité inspectée.
pub struct UiDataContext<'a> {
    pub entity: Entity,
    pub world: &'a World,
}

impl<'a> UiDataContext<'a> {
    pub fn new(entity: Entity, world: &'a World) -> Self {
        Self { entity, world }
    }

    /// Résout une clé de donnée en String.
    /// Format: "component.field" (ex: "inventory.capacity")
    /// Si la clé ne contient pas de point, cherche un composant dont le nom correspond.
    pub fn resolve(&self, key: &str) -> String {
        match key {
            "entity.id" => format!("{}", self.entity.to_bits()),
            "building.name" => self.get::<Building>().map(|b| b.name.clone()).unwrap_or_default(),
            "building.kind" => self.get::<Building>().map(|b| b.kind.clone()).unwrap_or_default(),
            "active" => self.get::<Active>().map(|a| if a.0 { "ON" } else { "OFF" }).unwrap_or("OFF").to_string(),
            "inventory.total" => self.get::<Inventory>().map(|i| i.total().to_string()).unwrap_or("0".to_string()),
            "inventory.capacity" => self.get::<Inventory>().map(|i| i.capacity.to_string()).unwrap_or("0".to_string()),
            _ => {
                // Essayer de trouver un composant avec ce nom
                // Fallback: retourner la clé elle-même
                key.to_string()
            }
        }
    }

    fn get<T: Component>(&self) -> Option<&'a T> {
        self.world.get::<T>(self.entity)
    }
}
