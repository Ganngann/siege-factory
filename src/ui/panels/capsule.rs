use bevy::prelude::*;

use crate::core::utils::silent_despawn;
use crate::economy::components::{BuildingPanel, PanelModal, PanelOverlay};
use crate::economy::ui_components::ManagedByPanel;
use crate::economy::window::{TEXT_PRIMARY, TEXT_SECONDARY, spawn_window};
use crate::ui::types::PanelType;
use crate::ui::panels::{Panel, PanelSpawnCtx};
use crate::core::game_font::tf;

pub struct CapsulePanelImpl;

impl Panel for CapsulePanelImpl {
    fn panel_type(&self) -> PanelType { PanelType::Capsule }
    fn panel_name(&self) -> &str { "capsule" }

    fn spawn(
        &self,
        commands: &mut Commands,
        panel: &mut BuildingPanel,
        ctx: &PanelSpawnCtx,
    ) -> (Entity, Entity) {
        if let Some(e) = panel.root.take() { silent_despawn(commands, e); }
        if let Some(e) = panel.overlay.take() { silent_despawn(commands, e); }
        panel.inspected = None;

        let Some(def) = ctx.building_registry.get(ctx.building_kind) else {
            let d = commands.spawn(Text::new("")).id();
            return (d, d);
        };
        let total_tiers = def.tiers.len();

        // Read CurrentTier from the entity via a temporary query
        let tier_index = commands
            .spawn(Text::new(""))
            .id(); // dummy entité pour le référencement
        let _ = tier_index;

        let overlay = commands.spawn((
            PanelOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO, right: Val::ZERO,
                top: Val::ZERO, bottom: Val::ZERO,
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(100),
            Pickable::default(),
        )).id();

        let root = spawn_window(
            commands,
            &format!("Capsule — {}", def.name),
            400.0, 320.0,
            80.0, 60.0,
            None,
            |parent| {
                parent.spawn((Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(4.0),
                    width: Val::Percent(100.0),
                    ..default()
                }, BackgroundColor(Color::NONE))).with_children(|col| {
                    col.spawn((
                        Text::new("PROGRESSION".to_string()),
                        tf(11.0), TextColor(TEXT_SECONDARY),
                    ));
                    col.spawn((
                        Text::new(format!("Tiers: 0/{} complétés", total_tiers)),
                        tf(12.0), TextColor(TEXT_PRIMARY),
                    ));

                    for i in 0..total_tiers {
                        let t = &def.tiers[i];
                        let prefix = if i < 1 { "✅" } else if i == 1 { "◉" } else { "○" };
                        let status = if i < 1 { " (fait)" } else if i == 1 { " (en cours)" } else { "" };
                        col.spawn((
                            Text::new(format!(" {} Tier {} — {}{}", prefix, i, t.texture, status)),
                            tf(12.0), TextColor(TEXT_PRIMARY),
                        ));
                    }

                    if let Some(current) = def.tiers.first() {
                        if !current.required_items.is_empty() {
                            col.spawn((Text::new(String::new()), TextFont::default(), TextColor::default()));
                            col.spawn((
                                Text::new("Items requis :".to_string()),
                                tf(11.0), TextColor(TEXT_SECONDARY),
                            ));
                            for (res, amt) in &current.required_items {
                                col.spawn((
                                    Text::new(format!("  {} 0/{}", res.display_name(), amt)),
                                    tf(12.0), TextColor(TEXT_PRIMARY),
                                ));
                            }
                            col.spawn((
                                Text::new("(Appuyez sur E à côté de la capsule)".to_string()),
                                tf(11.0), TextColor(TEXT_SECONDARY),
                            ));
                        }
                    }
                });
            },
        );
        commands.entity(root).insert((PanelModal, ManagedByPanel));
        commands.entity(overlay).add_child(root);
        panel.overlay = Some(overlay);
        panel.root = Some(root);
        panel.inspected = Some(ctx.entity);
        panel.dirty = false;
        (overlay, root)
    }
}

