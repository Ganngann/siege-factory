// Barrel re-export: all component types are defined in domain-specific modules
// but re-exported here so that existing `use crate::economy::components::X` paths
// continue to work without changes.
//
// 🪣 IA NOTE: si tu ajoutes un composant dans game_components.rs, power_components.rs,
// ui_components.rs ou discovery_components.rs, n'oublie PAS d'ajouter le `pub use` correspondant
// ici si tu veux qu'il soit accessible via `use crate::economy::components::NomDuComposant`.

pub use super::discovery_components::*;
pub use super::game_components::*;
pub use super::power_components::*;
pub use super::ui_components::*;
