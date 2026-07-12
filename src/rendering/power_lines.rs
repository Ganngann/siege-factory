// 🏗️ LEGACY UI — rendu des lignes électriques (gizmos).
// Pas de plan de migration pour ce fichier — il restera probablement un système à part.

use crate::economy::components::{PowerConsumer, PowerPole, UnbuiltBuilding};
use crate::economy::power_components::BurnerGenerator;
use bevy::prelude::*;

pub fn render_power_lines(
    poles: Query<(&PowerPole, &Transform), Without<UnbuiltBuilding>>,
    // SUGGEST: type ConsumerQuery = Query<(&PowerConsumer, &Transform), (Without<UnbuiltBuilding>, Without<BurnerGenerator>)> (clippy::type_complexity)
    consumers: Query<
        (&PowerConsumer, &Transform),
        (Without<UnbuiltBuilding>, Without<BurnerGenerator>),
    >,
    mut gizmos: Gizmos,
) {
    let pole_data: Vec<(Vec3, f32)> = poles
        .iter()
        .map(|(p, tf)| (tf.translation, p.range))
        .collect();

    if pole_data.is_empty() {
        return;
    }

    for (consumer, tf) in consumers.iter() {
        if !consumer.satisfied {
            continue;
        }
        let pos = tf.translation;
        if let Some(&(pp, _)) = pole_data
            .iter()
            .find(|(pp, range)| pp.distance(pos) <= *range)
        {
            // ⚠️ IA ATTENTION: couleur de ligne électrique en dur.
            gizmos.line(pp, pos, Color::srgba(0.3, 0.6, 1.0, 0.6));
        }
    }
}
