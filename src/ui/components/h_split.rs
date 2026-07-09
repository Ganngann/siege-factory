use bevy::prelude::*;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct HSplitComponent;
impl UiComponent for HSplitComponent {
    fn id(&self) -> &str { "h_split" }
    fn render(&self, commands: &mut Commands, _parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let left_w = config.get("left_width").and_then(|v| v.as_float()).unwrap_or(58.0) as f32;
        let right_w = config.get("right_width").and_then(|v| v.as_float()).unwrap_or(38.0) as f32;
        let left_el = config.get("left").and_then(|v| v.as_array());
        let right_el = config.get("right").and_then(|v| v.as_array());
        let row = spawn_child(commands, _parent, (Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Row, flex_grow: 1.0, padding: UiRect::all(Val::Px(8.0)), ..default() },));
        let left = spawn_child(commands, row, (Node { width: Val::Percent(left_w), flex_direction: FlexDirection::Column, margin: UiRect::right(Val::Px(10.0)), ..default() },));
        if let Some(elements) = left_el { for el_cfg in elements { let cid = el_cfg.get("type").and_then(|v| v.as_str()).unwrap_or("label"); if let Some(comp) = registry.get(cid) { comp.render(commands, left, el_cfg, data, theme, registry); } } }
        let right = spawn_child(commands, row, (Node { width: Val::Percent(right_w), flex_direction: FlexDirection::Column, ..default() },));
        if let Some(elements) = right_el { for el_cfg in elements { let cid = el_cfg.get("type").and_then(|v| v.as_str()).unwrap_or("label"); if let Some(comp) = registry.get(cid) { comp.render(commands, right, el_cfg, data, theme, registry); } } }
        row
    }
}
