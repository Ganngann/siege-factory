use bevy::prelude::*;
use crate::ui::context::UiDataContext;
use crate::ui::theme::Theme;

/// Un composant UI réutilisable, paramétrable via TOML.
/// Reçoit tous ses paramètres directement pour éviter les conflits de borrow.
pub trait UiComponent: Send + Sync {
    fn id(&self) -> &str;
    fn render(
        &self,
        commands: &mut Commands,
        parent: Entity,
        config: &toml::Value,
        data: &UiDataContext,
        theme: &Theme,
        registry: &ComponentRegistry,
    ) -> Entity;
}

#[derive(Default)]
pub struct ComponentRegistry {
    pub components: Vec<Box<dyn UiComponent>>,
}

impl ComponentRegistry {
    pub fn register(&mut self, component: Box<dyn UiComponent>) {
        let id = component.id().to_string();
        info!("ComponentRegistry: registered '{}'", id);
        self.components.push(component);
    }

    pub fn get(&self, id: &str) -> Option<&dyn UiComponent> {
        self.components.iter().find(|c| c.id() == id).map(|c| c.as_ref())
    }
}

/// Helper pour spawner un enfant et l'attacher au parent.
pub fn spawn_child(commands: &mut Commands, parent: Entity, bundle: impl bevy::ecs::bundle::Bundle) -> Entity {
    let child = commands.spawn(bundle).id();
    commands.entity(parent).add_child(child);
    child
}
