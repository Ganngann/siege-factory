// 🏗️ LEGACY UI — système d'infobulles (tooltips).
// Pas encore migrée vers src/ui/. Le nouveau système n'a pas de composant tooltip équivalent.

use bevy::prelude::*;

use crate::core::game_font::tf;

#[derive(Resource, Default)]
pub struct TooltipText(pub Option<String>);

#[derive(Component)]
pub struct TooltipMarker;

pub fn tooltip_ui(
    tooltip: Res<TooltipText>,
    windows: Query<&Window>,
    mut text_query: Query<(Entity, &mut Text, &mut Node), With<TooltipMarker>>,
    mut commands: Commands,
) {
    if let Some(ref msg) = tooltip.0 {
        if let Ok(window) = windows.single() {
            if let Ok((_, mut text, mut style)) = text_query.single_mut() {
                **text = msg.clone();
                style.display = Display::Flex;
                if let Some(cursor) = window.cursor_position() {
                    style.left = Val::Px(cursor.x + 15.0);
                    style.top = Val::Px(cursor.y - 10.0);
                }
            } else {
                commands.spawn((
                    TooltipMarker,
                    Text::new(msg.as_str()),
                    tf(12.0),
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        display: Display::None,
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                ));
            }
        }
    } else if let Ok((_, _, mut style)) = text_query.single_mut() {
        style.display = Display::None;
    }
}
