use bevy::prelude::*;
use crate::economy::components::{InventoryGrid, InventorySlot};
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct InventoryGridComponent;
impl UiComponent for InventoryGridComponent {
    fn id(&self) -> &str { "inventory_grid" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let cols = config.get("cols").and_then(|v| v.as_integer()).unwrap_or(3) as usize;
        let rows = config.get("rows").and_then(|v| v.as_integer()).unwrap_or(2) as usize;
        let count = cols * rows; let slot_size = 40.0; let gap = 3.0;
        let container = spawn_child(commands, parent, (Node { flex_direction: FlexDirection::Column, ..default() }, BackgroundColor(Color::NONE)));
        commands.entity(container).with_children(|p| {
            p.spawn((Node { width: Val::Percent(100.0), height: Val::Px(12.0), ..default() }, BackgroundColor(theme.bar_bg)));
            p.spawn((InventoryGrid { cols, rows, owner: data.entity }, Node { width: Val::Px(cols as f32 * (slot_size + gap) + gap), padding: bevy::ui::UiRect::all(Val::Px(gap)), display: Display::Flex, flex_wrap: FlexWrap::Wrap, align_content: AlignContent::FlexStart, ..default() }, BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),)).with_children(|g| { for i in 0..count { g.spawn((InventorySlot { index: i }, Button, Node { width: Val::Px(slot_size), height: Val::Px(slot_size), flex_shrink: 0.0, margin: bevy::ui::UiRect::axes(Val::Px(gap / 2.0), Val::Px(gap / 2.0)), border: bevy::ui::UiRect::all(Val::Px(1.0)), display: Display::Flex, flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }, BorderColor::all(Color::srgba(0.3, 0.3, 0.4, 1.0)), BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 1.0)), Text::new(String::new()), tf(9.0), TextColor(Color::WHITE))); } });
        });
        container
    }
}
