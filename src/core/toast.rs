use bevy::prelude::*;

use crate::rendering::config::VisualsConfig;

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
    config: Res<VisualsConfig>,
) {
    for msg in queue.0.drain(..) {
        commands.spawn((
            ToastMessage {
                timer: config.toast.lifetime,
            },
            Text::new(msg),
            TextFont::from_font_size(config.toast.font_size),
            TextColor(config.toast.color),
            TextLayout::justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(config.toast.bottom_px),
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


