use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

#[derive(Component)]
pub struct HoverableButton {
    pub inactive: Color,
    pub hover: Color,
    pub pressed: Color,
}

pub fn button_hover_system(
    mut query: Query<(&Interaction, &mut BackgroundColor, &HoverableButton), Changed<Interaction>>,
) {
    for (interaction, mut bg_color, hoverable) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(hoverable.pressed);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(hoverable.hover);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(hoverable.inactive);
            }
        }
    }
}

pub struct ButtonComponent;
impl UiComponent for ButtonComponent {
    fn id(&self) -> &str { "button" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let text = config.get("text").and_then(|v| v.as_str()).unwrap_or("Button").to_string();
        let child = spawn_child(commands, parent, (
            Button,
            HoverableButton {
                inactive: theme.btn_inactive,
                hover: theme.btn_hover,
                pressed: theme.btn_active,
            },
            Node { width: Val::Px(120.0), height: Val::Px(24.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::vertical(Val::Px(2.0)), ..default() },
            BackgroundColor(theme.btn_inactive)
        ));
        commands.entity(child).with_children(|p| { p.spawn((Text::new(text), tf(theme.font_size_body), TextColor(theme.text_primary))); });
        child
    }
}
