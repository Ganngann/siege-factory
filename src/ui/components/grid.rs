use bevy::prelude::*;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct GridComponent;
impl UiComponent for GridComponent {
    fn id(&self) -> &str { "grid" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &UiDataContext, theme: &Theme, registry: &ComponentRegistry) -> Entity {
        let total_cols = config.get("cols").and_then(|v| v.as_integer()).unwrap_or(1) as f32;
        let gap = config.get("gap").and_then(|v| v.as_float()).unwrap_or(8.0) as f32;
        let children = config.get("children").and_then(|v| v.as_array());

        let container = spawn_child(commands, parent, (
            Node {
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                column_gap: Val::Px(gap),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));

        if let Some(arr) = children {
            for child_config in arr {
                let ctype = child_config.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if ctype == "column" {
                    let col_width = child_config.get("width").and_then(|v| v.as_integer()).unwrap_or(1) as f32;
                    let pct = (col_width / total_cols * 100.0).max(5.0);
                    let col_container = spawn_child(commands, container, (
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(pct),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ));
                    if let Some(col_children) = child_config.get("children").and_then(|v| v.as_array()) {
                        for cc in col_children {
                            let cid = cc.get("type").and_then(|v| v.as_str()).unwrap_or("label");
                            if let Some(comp) = registry.get(cid) {
                                comp.render(commands, col_container, cc, data, theme, registry);
                            }
                        }
                    }
                }
            }
        }

        container
    }
}
