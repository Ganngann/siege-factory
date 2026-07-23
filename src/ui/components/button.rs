use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::registry::{UiComponent, spawn_child};

#[derive(Component)]
pub struct HoverableButton {
    pub inactive: Color,
    pub hover: Color,
    pub pressed: Color,
}

pub struct ButtonComponent;
impl UiComponent for ButtonComponent {
    fn id(&self) -> &str { "button" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &crate::ui::context::UiDataContext, theme: &crate::ui::theme::Theme, _registry: &crate::ui::registry::ComponentRegistry) -> Entity {
        let text = config.get("text").and_then(|v| v.as_str()).unwrap_or("Button").to_string();

        let hoverable = HoverableButton {
            inactive: theme.btn_inactive,
            hover: theme.btn_hover,
            pressed: theme.btn_active,
        };

        let child = spawn_child(commands, parent, (
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme.btn_inactive),
            hoverable
        ));

        commands.entity(child).with_children(|p| {
            p.spawn((Text::new(text), tf(theme.font_size_body), TextColor(theme.text_primary)));
        });

        child
    }
}

pub fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &HoverableButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, hoverable) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = hoverable.pressed.into();
            }
            Interaction::Hovered => {
                *color = hoverable.hover.into();
            }
            Interaction::None => {
                *color = hoverable.inactive.into();
            }
        }
    }
}
