use crate::economy::tiered_structure::ProgressionLogRegistry;
use crate::ui::components::data_list::DataListSelected;
use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct DataTextComponent;
impl UiComponent for DataTextComponent {
    fn id(&self) -> &str { "data_text" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let data_key = config.get("data_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
        spawn_child(commands, parent, (
            Text::new("(Select a log)".to_string()),
            tf(theme.font_size_body),
            TextColor(Color::srgb(0.60, 0.60, 0.75)),
            DataText { data_key },
        ))
    }
}

#[derive(Component, Clone)]
pub struct DataText {
    pub data_key: String,
}

pub fn update_data_text_system(
    mut q: Query<(&DataText, &mut Text)>,
    selected: Res<DataListSelected>,
    logs: Res<ProgressionLogRegistry>,
) {
    if !selected.is_changed() { return; }
    for (dt, mut text) in &mut q {
        if dt.data_key != "capsule.log_text" { continue; }
        let Some(log) = logs.logs.iter().find(|l| l.id == selected.selected_id) else { continue; };
        text.0 = log.text.clone();
    }
}
