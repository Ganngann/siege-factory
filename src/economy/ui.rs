use bevy::prelude::*;
use crate::economy::resource::{ResourceRegistry, Inventory};
use crate::economy::components::HQ;

pub fn resource_count_ui(
    hq_query: Query<&Inventory, With<HQ>>,
    mut text_query: Query<(Entity, &mut Text), With<ResourceCountText>>,
    mut commands: Commands,
    registry: Res<ResourceRegistry>,
) {
    let msg = if let Ok(inv) = hq_query.single() {
        let mut parts = Vec::new();
        for (res_id, amount) in &inv.resources {
            if let Some(def) = registry.get_opt(&res_id.0) {
                parts.push(format!("{}: {}", def.name, amount));
            } else {
                parts.push(format!("{}: {}", res_id.display_name(), amount));
            }
        }
        parts.join("  ")
    } else {
        String::new()
    };

    if let Ok((_, mut text)) = text_query.single_mut() {
        text.0 = msg;
    } else if !msg.is_empty() {
        commands.spawn((
            ResourceCountText,
            Text::new(msg),
            TextFont::from_font_size(18.0),
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
        ));
    }
}

#[derive(Component)]
pub struct ResourceCountText;
