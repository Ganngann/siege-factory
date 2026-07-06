use bevy::prelude::*;

const TOAST_LIFETIME: f32 = 3.0;

#[derive(Component)]
pub struct ToastMessage {
    pub timer: f32,
}

#[derive(Resource, Default)]
pub struct ToastQueue(pub Vec<String>);

pub fn toast_system(
    mut commands: Commands,
    mut queue: ResMut<ToastQueue>,
    time: Res<Time>,
    mut toasts: Query<(Entity, &mut ToastMessage)>,
) {
    for msg in queue.0.drain(..) {
        commands.spawn((
            ToastMessage { timer: TOAST_LIFETIME },
            Text::new(msg),
            TextFont::from_font_size(16.0),
            TextColor(Color::srgb(1.0, 0.85, 0.3)),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(90.0),
                left: Val::Percent(50.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));
    }
    for (entity, mut msg) in toasts.iter_mut() {
        msg.timer -= time.delta_secs();
        if msg.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}
