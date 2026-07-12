use bevy::prelude::*;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct BuildBarComponent;
impl UiComponent for BuildBarComponent {
    fn id(&self) -> &str { "build_bar" }
    fn render(&self, commands: &mut Commands, parent: Entity, _config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let child = spawn_child(commands, parent, (
            crate::economy::components::MenuBarPanel,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(90.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
            Pickable::default(),
        ));
        child
    }
}
