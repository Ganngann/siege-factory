// Barrel re-export: all component types are defined in domain-specific modules
// but re-exported here so that existing `use crate::economy::components::X` paths
// continue to work without changes.

pub use super::discovery_components::*;
pub use super::game_components::*;
pub use super::power_components::*;
pub use super::ui_components::*;
