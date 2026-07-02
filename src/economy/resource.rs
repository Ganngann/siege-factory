use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceId {
    Ore,
    Ammo,
    Energy,
}

impl ResourceId {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ore" => Some(Self::Ore),
            "ammo" => Some(Self::Ammo),
            "energy" => Some(Self::Energy),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            ResourceId::Ore => "Ore",
            ResourceId::Ammo => "Ammo",
            ResourceId::Energy => "Energy",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceDef {
    pub id: ResourceId,
    pub name: String,
    pub max_stack: u32,
}

#[derive(Debug, Clone, Resource)]
pub struct ResourceRegistry {
    pub resources: HashMap<ResourceId, ResourceDef>,
}

impl ResourceRegistry {
    pub fn load() -> Self {
        let toml_str = include_str!("../../data/resources.toml");
        let parsed: ResourcesToml = toml::from_str(toml_str).expect("failed to parse resources.toml");
        let mut resources = HashMap::new();
        for (key, entry) in parsed.resources {
            if let Some(id) = ResourceId::from_str(&key) {
                resources.insert(id, ResourceDef {
                    id,
                    name: entry.name,
                    max_stack: entry.max_stack,
                });
            }
        }
        Self { resources }
    }

    pub fn get(&self, id: ResourceId) -> &ResourceDef {
        &self.resources[&id]
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
}

#[derive(Debug, Clone, Component)]
pub struct Inventory {
    pub resources: HashMap<ResourceId, u32>,
    pub capacity: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self { resources: HashMap::new(), capacity: 0 }
    }

    pub fn with_capacity(capacity: u32) -> Self {
        Self { resources: HashMap::new(), capacity }
    }

    pub fn get(&self, resource: ResourceId) -> u32 {
        *self.resources.get(&resource).unwrap_or(&0)
    }

    pub fn add(&mut self, resource: ResourceId, amount: u32) {
        let entry = self.resources.entry(resource).or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    pub fn try_add(&mut self, resource: ResourceId, amount: u32) -> bool {
        if self.capacity > 0 && self.total() + amount > self.capacity {
            return false;
        }
        self.add(resource, amount);
        true
    }

    pub fn remove(&mut self, resource: ResourceId, amount: u32) -> bool {
        let entry = self.resources.entry(resource).or_insert(0);
        if *entry >= amount {
            *entry -= amount;
            true
        } else {
            false
        }
    }

    pub fn total(&self) -> u32 {
        self.resources.values().sum()
    }

    pub fn is_full(&self) -> bool {
        self.capacity > 0 && self.total() >= self.capacity
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_inventory_is_empty() {
        let inv = Inventory::new();
        assert_eq!(inv.get(ResourceId::Ore), 0);
        assert_eq!(inv.get(ResourceId::Ammo), 0);
    }

    #[test]
    fn add_increases_amount() {
        let mut inv = Inventory::new();
        inv.add(ResourceId::Ore, 10);
        assert_eq!(inv.get(ResourceId::Ore), 10);
    }

    #[test]
    fn add_stacks_multiple_times() {
        let mut inv = Inventory::new();
        inv.add(ResourceId::Ore, 5);
        inv.add(ResourceId::Ore, 7);
        assert_eq!(inv.get(ResourceId::Ore), 12);
    }

    #[test]
    fn remove_reduces_amount() {
        let mut inv = Inventory::new();
        inv.add(ResourceId::Ore, 10);
        assert!(inv.remove(ResourceId::Ore, 4));
        assert_eq!(inv.get(ResourceId::Ore), 6);
    }

    #[test]
    fn remove_returns_false_if_not_enough() {
        let mut inv = Inventory::new();
        inv.add(ResourceId::Ore, 3);
        assert!(!inv.remove(ResourceId::Ore, 5));
        assert_eq!(inv.get(ResourceId::Ore), 3);
    }

    #[test]
    fn add_never_overflows() {
        let mut inv = Inventory::new();
        inv.add(ResourceId::Ore, u32::MAX);
        inv.add(ResourceId::Ore, 1);
        assert_eq!(inv.get(ResourceId::Ore), u32::MAX);
    }

    #[test]
    fn different_resources_independent() {
        let mut inv = Inventory::new();
        inv.add(ResourceId::Ore, 10);
        inv.add(ResourceId::Ammo, 5);
        assert_eq!(inv.get(ResourceId::Ore), 10);
        assert_eq!(inv.get(ResourceId::Ammo), 5);
        assert_eq!(inv.get(ResourceId::Energy), 0);
    }

    #[test]
    fn capacity_limits_add() {
        let mut inv = Inventory::with_capacity(10);
        assert!(inv.add(ResourceId::Ore, 5));
        assert!(inv.add(ResourceId::Ore, 5));
        assert!(!inv.add(ResourceId::Ore, 1));
        assert_eq!(inv.get(ResourceId::Ore), 10);
    }

    proptest::proptest! {
        #[test]
        fn inventory_never_negative(
            add1 in 0..1000u32,
            add2 in 0..1000u32,
            remove in 0..2000u32,
        ) {
            let mut inv = Inventory::new();
            inv.add(ResourceId::Ore, add1);
            inv.add(ResourceId::Ore, add2);
            let before = inv.get(ResourceId::Ore);
            inv.remove(ResourceId::Ore, remove);
            assert!(inv.get(ResourceId::Ore) <= before);
        }
    }
}
