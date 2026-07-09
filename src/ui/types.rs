use bevy::prelude::*;

use crate::core::game_font::tf;

// ── Panel types (used by PanelRegistry and legacy panels) ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelType {
    Building,
    Capsule,
    Deposit,
    Crafting,
    Pause,
    MainMenu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Title,
    Body,
    Small,
    Accent,
    Monospace,
}

// ── Components ──

/// Marqueur : cette entité est un panneau.
#[derive(Component)]
pub struct Panel {
    pub panel_type: PanelType,
    pub dirty: bool,
}

/// Marqueur : section d'un panneau.
#[derive(Component)]
pub struct PanelSection;

/// Marqueur : le panneau est géré par le PanelManager.
#[derive(Component)]
pub struct ManagedPanel;

// ── Helper functions ──

pub fn styled_label(text: &str, style: LabelStyle) -> impl Bundle {
    let (size, color) = match style {
        LabelStyle::Title => (16.0, Color::srgb(0.90, 0.90, 1.00)),
        LabelStyle::Body => (12.0, Color::srgb(0.60, 0.60, 0.75)),
        LabelStyle::Small => (10.0, Color::srgb(0.50, 0.50, 0.65)),
        LabelStyle::Accent => (12.0, Color::srgb(0.30, 0.55, 1.00)),
        LabelStyle::Monospace => (12.0, Color::srgb(0.90, 0.90, 1.00)),
    };
    (
        Text::new(text.to_string()),
        tf(size),
        TextColor(color),
    )
}
