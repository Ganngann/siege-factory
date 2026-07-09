use bevy::prelude::*;

use crate::core::game_font::GameFont;
use crate::core::utils::silent_despawn;
use crate::rendering::config::VisualsConfig;

#[derive(Component)]
pub struct ToastMessage {
    pub timer: f32,
    pub persistent: bool,
}

/// Entrée dans la file d'attente des toasts.
pub struct ToastEntry {
    pub text: String,
    pub persistent: bool,
}

#[derive(Resource, Default)]
pub struct ToastQueue(pub Vec<String>);

impl ToastQueue {
    /// Ajoute un toast qui disparaît automatiquement après lifetime.
    pub fn push(&mut self, msg: impl Into<String>) {
        self.0.push(msg.into());
    }

    /// Ajoute un toast qui reste à l'écran jusqu'à dismiss par le joueur.
    pub fn push_persistent(&mut self, msg: impl Into<String>) {
        self.0.push(format!("\x00PERSISTENT\x00{}", msg.into()));
    }
}

pub fn toast_system(
    mut commands: Commands,
    mut queue: ResMut<ToastQueue>,
    time: Res<Time>,
    mut toasts: Query<(Entity, &mut ToastMessage)>,
    config: Res<VisualsConfig>,
    font: Res<GameFont>,
) {
    for msg in queue.0.drain(..) {
        let persistent = msg.starts_with("\x00PERSISTENT\x00");
        let text = if persistent { &msg[14..] } else { &msg };

        commands.spawn((
            ToastMessage {
                timer: config.toast.lifetime,
                persistent,
            },
            Text::new(text.to_string()),
            TextFont {
                font: font.0.clone().into(),
                font_size: config.toast.font_size.into(),
                ..default()
            },
            TextColor(config.toast.color),
            TextLayout::new(Justify::Center, bevy::text::LineBreak::WordBoundary),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(config.toast.bottom_px),
                left: Val::Auto,
                right: Val::Auto,
                justify_content: JustifyContent::Center,
                max_width: Val::Px(500.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
        ));
    }
    for (entity, mut msg) in toasts.iter_mut() {
        if msg.persistent {
            continue;
        }
        msg.timer -= time.delta_secs();
        if msg.timer <= 0.0 {
            silent_despawn(&mut commands, entity);
        }
    }
}

/// Dismiss les toasts persistants quand le joueur clique ou appuie sur Espace.
pub fn dismiss_persistent_toasts(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    toasts: Query<(Entity, &ToastMessage)>,
) {
    if !buttons.just_pressed(MouseButton::Left) && !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for (entity, msg) in toasts.iter() {
        if msg.persistent {
            silent_despawn(&mut commands, entity);
        }
    }
}


