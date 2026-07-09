use bevy::prelude::*;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct VStackComponent;
impl UiComponent for VStackComponent {
    fn id(&self) -> &str { "v_stack" }
    fn render(&self, commands: &mut Commands, _parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let children = config.get("children").and_then(|v| v.as_array());
        let container = spawn_child(commands, _parent, (Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },));
        if let Some(items) = children { for child_cfg in items { let cid = child_cfg.get("type").and_then(|v| v.as_str()).unwrap_or("label"); if let Some(comp) = registry.get(cid) { comp.render(commands, container, child_cfg, data, theme, registry); } } }
        container
    }
}
