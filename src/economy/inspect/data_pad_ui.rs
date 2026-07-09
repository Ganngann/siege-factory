use bevy::prelude::*;

use crate::economy::tiered_structure::ProgressionLogRegistry;
use crate::economy::ui_components::{DataPadEntry, DataPadFullText, DataPadSelected};

/// Détecte les clics sur les entrées du Data Pad et met à jour l'affichage.
pub fn data_pad_select_log(
    mut selected: ResMut<DataPadSelected>,
    mut full_text_q: Query<&mut Text, (With<DataPadFullText>, Without<DataPadEntry>)>,
    buttons: Query<(&Interaction, &DataPadEntry), Changed<Interaction>>,
    logs: Res<ProgressionLogRegistry>,
) {
    for (interaction, entry) in buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        selected.log_id = Some(entry.log_id.clone());

        // Update the full text display
        if let Some(log) = logs.logs.iter().find(|l| l.id == entry.log_id) {
            for mut text in full_text_q.iter_mut() {
                text.0 = format!("\"{}\"\n\nTier {} — {}", log.text, log.tier, log.title);
            }
        }
    }
}
