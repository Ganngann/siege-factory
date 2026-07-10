use crate::economy::tiered_structure::ProgressionLogRegistry;
use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct DataListComponent;
impl UiComponent for DataListComponent {
    fn id(&self) -> &str { "data_list" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let data_key = config.get("data_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
        spawn_child(commands, parent, (
            Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::NONE),
            DataList { data_key },
        ))
    }
}

#[derive(Component, Clone)]
pub struct DataList {
    pub data_key: String,
}

#[derive(Component, Clone)]
pub struct DataListItem {
    pub list_key: String,
    pub item_id: String,
}

#[derive(Resource, Default)]
pub struct DataListSelected {
    pub list_key: String,
    pub selected_id: String,
}

pub fn populate_data_list(
    mut commands: Commands,
    q: Query<(Entity, &DataList), Added<DataList>>,
    logs: Res<ProgressionLogRegistry>,
) {
    for (entity, list) in &q {
        if list.data_key != "capsule.logs" { continue; }
        commands.entity(entity).with_children(|parent| {
            for log in &logs.logs {
                if !logs.unlocked.contains(&log.id) { continue; }
                let is_done = true; // all unlocked logs are done
                let prefix = if is_done { "●" } else { "○" };
                parent.spawn((
                    DataListItem {
                        list_key: list.data_key.clone(),
                        item_id: log.id.clone(),
                    },
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(24.0),
                        padding: UiRect::horizontal(Val::Px(4.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.15, 1.0)),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("{}  {} (Tier {})", prefix, log.title, log.tier)),
                        tf(12.0),
                        TextColor(Color::srgb(0.60, 0.60, 0.75)),
                    ));
                });
            }
        });
    }
}

pub fn data_list_click_system(
    mut selected: ResMut<DataListSelected>,
    q: Query<(&Interaction, &DataListItem), Changed<Interaction>>,
) {
    for (interaction, item) in &q {
        if *interaction != Interaction::Pressed { continue; }
        selected.list_key = item.list_key.clone();
        selected.selected_id = item.item_id.clone();
    }
}
