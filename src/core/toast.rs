use bevy::prelude::*;

use crate::core::utils::silent_despawn;

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

/// Dismiss les toasts persistants quand le joueur appuie sur Espace.
pub fn dismiss_persistent_toasts(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    toasts: Query<(Entity, &ToastMessage)>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for (entity, msg) in toasts.iter() {
        if msg.persistent {
            silent_despawn(&mut commands, entity);
        }
    }
}


