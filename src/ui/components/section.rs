use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct SectionComponent;
impl UiComponent for SectionComponent {
    fn id(&self) -> &str { "section" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let title = config.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let section = spawn_child(commands, parent, (Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(8.0)), margin: UiRect::bottom(Val::Px(6.0)), ..default() }, BackgroundColor(theme.section_bg)));
        if !title.is_empty() {
            commands.entity(section).with_children(|s| { s.spawn((Text::new(title), tf(theme.font_size_label), TextColor(theme.text_secondary), Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() })); });
        }
        if let Some(elements) = config.get("elements").and_then(|v| v.as_array()) {
            for el_cfg in elements {
                let cid = el_cfg.get("type").and_then(|v| v.as_str()).unwrap_or("label");
                if let Some(comp) = registry.get(cid) {
                    comp.render(commands, section, el_cfg, data, theme, registry);
                }
            }
        }
        section
    }
}
