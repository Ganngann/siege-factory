use bevy::prelude::*;
use crate::economy::resource::{ResourceId, Inventory};
use crate::economy::components::HQ;

pub fn ore_count_ui(
    hq_query: Query<&Inventory, With<HQ>>,
    mut text_query: Query<(Entity, &mut Text), With<OreCountText>>,
    mut commands: Commands,
) {
    let ore = hq_query
        .single()
        .map(|inv| inv.get(ResourceId::Ore))
        .unwrap_or(0);
    let ammo = hq_query
        .single()
        .map(|inv| inv.get(ResourceId::Ammo))
        .unwrap_or(0);
    let energy = hq_query
        .single()
        .map(|inv| inv.get(ResourceId::Energy))
        .unwrap_or(0);

    let msg = format!("Ore: {ore}  Ammo: {ammo}  Energy: {energy}");

    if let Ok((_, mut text)) = text_query.single_mut() {
        text.0 = msg;
    } else {
        commands.spawn((
            OreCountText,
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
pub struct OreCountText;
