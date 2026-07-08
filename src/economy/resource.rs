use crate::core::utils::parse_hex_color;
use crate::load_toml;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into().to_lowercase())
    }

    pub fn display_name(&self) -> String {
        self.0
            .split('_')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub resource: ResourceId,
    pub amount: u32,
}

#[derive(Debug, Clone)]
pub struct ResourceDef {
    pub id: String,
    pub name: String,
    pub max_stack: u32,
    pub color: Color,
}

#[derive(Debug, Clone, Resource)]
pub struct ResourceRegistry {
    pub resources: HashMap<String, ResourceDef>,
}

impl ResourceRegistry {
    pub fn load() -> Self {
        let parsed: ResourcesToml = load_toml!("../../data/resources.toml", ResourcesToml);
        let mut resources = HashMap::new();
        for (key, entry) in parsed.resources {
            resources.insert(
                key.clone(),
                ResourceDef {
                    id: key,
                    name: entry.name,
                    max_stack: entry.max_stack,
                    color: entry
                        .color
                        .as_deref()
                        .map(parse_hex_color)
                        .unwrap_or(Color::srgb(0.5, 0.5, 0.5)),
                },
            );
        }
        Self { resources }
    }

    pub fn get(&self, id: &str) -> &ResourceDef {
        &self.resources[id]
    }

    pub fn get_opt(&self, id: &str) -> Option<&ResourceDef> {
        self.resources.get(id)
    }

    pub fn display_name<'a>(&'a self, id: &'a ResourceId) -> &'a str {
        match self.resources.get(&id.0) {
            Some(r) => &r.name,
            None => &id.0,
        }
    }

    pub fn apply_mod_overrides(&mut self, mods: &crate::core::modding::ModRegistry) {
        let Some(content) = mods.load_data("resources.toml") else {
            return;
        };
        let Ok(parsed) = toml::from_str::<ResourcesToml>(&content) else {
            bevy::prelude::error!("Failed to parse resources.toml from mod");
            return;
        };
        for (id, entry) in parsed.resources {
            self.resources.insert(
                id.clone(),
                ResourceDef {
                    id,
                    name: entry.name,
                    max_stack: entry.max_stack,
                    color: entry
                        .color
                        .as_deref()
                        .map(parse_hex_color)
                        .unwrap_or(Color::srgb(0.5, 0.5, 0.5)),
                },
            );
        }
    }
}

#[derive(Deserialize)]
struct ResourcesToml {
    resources: HashMap<String, ResourceEntry>,
}

#[derive(Deserialize)]
struct ResourceEntry {
    name: String,
    max_stack: u32,
    color: Option<String>,
}

// ── Slot-based Inventory ──

/// Slot-based inventory. Each slot holds one resource type with an amount,
/// or is `None` (empty). `capacity` limits the **total** item count across
/// all slots (0 = unlimited). `add` auto-grows the slot Vec when no matching
/// or empty slot is available, so the inventory is always usable — the grid
/// simply displays the first N slots.
#[derive(Debug, Clone, Default, Component)]
pub struct Inventory {
    pub(crate) slots: Vec<Option<(ResourceId, u32)>>,
    pub(crate) capacity: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: u32) -> Self {
        Self {
            slots: Vec::new(),
            capacity,
        }
    }

    pub fn with_slots(slot_count: usize, capacity: u32) -> Self {
        Self {
            slots: vec![None; slot_count],
            capacity,
        }
    }

    /// Total amount of a given resource across all slots.
    pub fn get(&self, resource: &ResourceId) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|(r, _)| r == resource)
            .map(|(_, a)| a)
            .sum()
    }

    /// Add items. The resource is placed into an existing matching slot, or
    /// the first empty slot, or a new slot is appended (auto-grow).
    pub fn add(&mut self, resource: &ResourceId, amount: u32) {
        // 1. Matching slot
        for slot in self.slots.iter_mut() {
            if let Some((r, a)) = slot {
                if *r == *resource {
                    *a = a.saturating_add(amount);
                    return;
                }
            }
        }
        // 2. Empty slot
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some((resource.clone(), amount));
                return;
            }
        }
        // 3. Auto-grow
        self.slots.push(Some((resource.clone(), amount)));
    }

    /// Add only if there is room (`capacity`) AND a matching or empty slot
    /// exists (or can be auto-grown — this always succeeds within capacity).
    pub fn try_add(&mut self, resource: &ResourceId, amount: u32) -> bool {
        if self.capacity > 0 && self.total() + amount > self.capacity {
            return false;
        }
        self.add(resource, amount);
        true
    }

    /// Remove `amount` of `resource` from the **first** matching slot.
    /// Returns `false` if the resource doesn't exist or the amount is
    /// insufficient.
    pub fn remove(&mut self, resource: &ResourceId, amount: u32) -> bool {
        for slot in self.slots.iter_mut() {
            if let Some((r, a)) = slot {
                if *r == *resource && *a >= amount {
                    *a -= amount;
                    if *a == 0 {
                        *slot = None;
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Total items across all slots.
    pub fn total(&self) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|(_, a)| a)
            .sum()
    }

    /// Full when `capacity > 0` and total ≥ capacity.
    pub fn is_full(&self) -> bool {
        self.capacity > 0 && self.total() >= self.capacity
    }

    /// Number of slots (including empty).
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Number of non‑empty slots.
    pub fn occupied_slot_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Whether the slot at `index` exists and is non‑empty.
    pub fn is_empty_slot(&self, index: usize) -> bool {
        self.slots.get(index).map_or(true, |s| s.is_none())
    }

    /// Content of the slot at `index`.
    pub fn slot_content(&self, index: usize) -> Option<&(ResourceId, u32)> {
        self.slots.get(index).and_then(|s| s.as_ref())
    }

    /// Mutable content of the slot at `index`.
    pub fn slot_content_mut(&mut self, index: usize) -> Option<&mut (ResourceId, u32)> {
        self.slots.get_mut(index).and_then(|s| s.as_mut())
    }

    /// Swap the contents of two slots. Auto‑grows the Vec if needed.
    pub fn swap_slots(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let max = a.max(b);
        if max >= self.slots.len() {
            self.slots.resize(max + 1, None);
        }
        self.slots.swap(a, b);
    }

    /// The first occupied slot's resource, if any.
    pub fn first_resource(&self) -> Option<ResourceId> {
        self.slots
            .iter()
            .find_map(|s| s.as_ref().map(|(r, _)| r.clone()))
    }

    /// Iterate over all occupied slots.
    pub fn iter_occupied(&self) -> impl Iterator<Item = &(ResourceId, u32)> {
        self.slots.iter().filter_map(|s| s.as_ref())
    }

    /// Clear all slots (set every slot to `None`).
    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut() {
            *slot = None;
        }
    }

    // ── Serialisation helpers ──

    pub fn to_raw_slots(&self) -> Vec<Option<(String, u32)>> {
        self.slots
            .iter()
            .map(|s| s.as_ref().map(|(r, a)| (r.0.clone(), *a)))
            .collect()
    }

    pub fn from_raw_slots(
        raw: Vec<Option<(String, u32)>>,
        capacity: u32,
    ) -> Self {
        Self {
            slots: raw
                .into_iter()
                .map(|s| s.map(|(r, a)| (ResourceId(r), a)))
                .collect(),
            capacity,
        }
    }
}


