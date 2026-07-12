pub mod building;
pub mod capsule;

use bevy::prelude::*;

use crate::core::modding::ModRegistry;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::BuildingPanel;
use crate::economy::resource::ResourceRegistry;
use crate::ui::context::UiDataContext;
use crate::ui::engine::LayoutEngine;
use crate::ui::types::PanelType;

/// Contexte passé à `Panel::spawn()` — toutes les ressources disponibles
/// au moment du clic inspect.
pub struct PanelSpawnCtx<'a> {
    pub entity: Entity,
    pub building_kind: &'a str,
    pub building_registry: &'a BuildingRegistry,
    pub resource_registry: &'a ResourceRegistry,
    pub data: &'a UiDataContext,
    pub mods: &'a ModRegistry,
    pub layout_engine: &'a LayoutEngine,
}

/// Cycle de vie standardisé d'un panneau UI.
/// Chaque implémentation gère son propre spawn/close/update.
pub trait Panel: Send + Sync {
    fn panel_type(&self) -> PanelType;
    fn panel_name(&self) -> &str;

    /// Crée l'entité panneau et retourne (overlay_id, root_id).
    fn spawn(
        &self,
        commands: &mut Commands,
        panel: &mut BuildingPanel,
        ctx: &PanelSpawnCtx,
    ) -> (Entity, Entity);

    /// Met à jour le contenu du panneau (quand dirty = true).
    fn update(&self, _commands: &mut Commands, _panel: &BuildingPanel) {}

    /// Nettoie le panneau.
    fn close(&self, commands: &mut Commands, panel: &mut BuildingPanel) {
        use crate::core::utils::silent_despawn;
        if let Some(e) = panel.root.take() {
            silent_despawn(commands, e);
        }
        if let Some(e) = panel.overlay.take() {
            silent_despawn(commands, e);
        }
        panel.inspected = None;
        panel.dirty = false;
    }
}

/// Registre des types de panneaux. Accessible par les mods pour enregistrer leurs propres panneaux.
#[derive(Resource, Default)]
pub struct PanelRegistry {
    pub panels: Vec<Box<dyn Panel>>,
}

impl PanelRegistry {
    pub fn register(&mut self, panel: Box<dyn Panel>) {

        self.panels.push(panel);
    }

    pub fn get(&self, panel_type: &PanelType) -> Option<&dyn Panel> {
        self.panels.iter().find(|p| p.panel_type() == *panel_type).map(|p| p.as_ref())
    }
}

