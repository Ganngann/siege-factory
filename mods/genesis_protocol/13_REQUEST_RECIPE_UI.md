# Request 13 — UI de craft invisible pour les bâtiments à recettes

## Contexte

Le système d'inspection des bâtiments utilise une **liste codée en dur** dans `src/economy/inspect/spawn.rs` (fonction `kind_has_recipes`) pour décider si un bâtiment doit afficher l'interface de sélection de recettes ou simplement l'inventaire.

Les bâtiments du mod Genesis Protocol (workbench, anvil, water_pump, steam_generator, gear_press, chemical_lab, motor_foundry, battery_station, electronics_lab, nanite_assembler, bio_lab, tissue_cultivator, synthesizer, scanner_array, bio_printer) ne sont pas dans cette liste → le joueur voit une fenêtre inventaire vide sans possibilité de crafter.

## Solution demandée

Ajouter un champ optionnel `has_recipes: bool` à `BuildingDef` dans le TOML, et remplacer la fonction hardcodée par une lecture de ce champ.

### Étapes

**1. Rust — modifier `BuildingDef`** (`src/economy/building.rs`) :

```rust
// Ajouter dans la struct BuildingDef
pub has_recipes: bool,
```

Avec `#[serde(default)]` pour la rétrocompatibilité (les mods qui n'ont pas ce champ continuent de fonctionner).

**2. Rust — remplacer `kind_has_recipes`** (`src/economy/inspect/spawn.rs`) :

```rust
// Ancien code (hardcodé) :
fn kind_has_recipes(kind: &str) -> bool {
    matches!(kind, "assembler" | "furnace" | ...)
}

// Nouveau code (data-driven) :
fn kind_has_recipes(kind: &str, registry: &BuildingRegistry) -> bool {
    registry.get(kind).map(|def| def.has_recipes).unwrap_or(false)
}
```

**3. TOML — ajouter `has_recipes = true`** à tous les bâtiments qui ont `recipe_categories` non vides dans `mods/genesis_protocol/data/buildings.toml`.

### Bâtiments concernés dans notre mod

Tous ceux avec `recipe_categories` défini :
- workbench, campfire, furnace, anvil, water_pump, steam_generator
- blast_furnace, gear_press, assembler, chemical_lab
- motor_foundry, battery_station, electronics_lab
- assembly_crane, nanite_assembler, bio_lab, tissue_cultivator
- synthesizer, scanner_array, bio_printer

## Résultat attendu

- Le joueur clique sur un établis → voit l'interface de sélection de recettes (pas l'inventaire vide)
- Les bâtiments sans `has_recipes` (défaut `false`) continuent d'afficher l'inventaire
- Rétrocompatible avec les mods existants
