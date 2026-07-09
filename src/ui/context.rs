use std::collections::HashMap;

use bevy::prelude::*;

/// Résout les clés de données comme "building.name" ou "health.current"
/// en valeurs String à partir d'une HashMap pré-remplie au moment du clic.
/// Plus besoin de &World — les données sont résolues en amont.
pub struct UiDataContext {
    pub entity: Entity,
    pub data: HashMap<String, String>,
}

impl UiDataContext {
    pub fn new(entity: Entity, data: HashMap<String, String>) -> Self {
        Self { entity, data }
    }

    /// Résout une clé de donnée en String.
    /// Si la clé n'existe pas, retourne une chaîne vide.
    pub fn resolve(&self, key: &str) -> String {
        self.data.get(key).cloned().unwrap_or_default()
    }
}
