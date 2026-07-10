# Request 20 — Rendre build.rs data-driven (supprimer le if/else par ID)

## Problème

Actuellement `src/economy/placement/build.rs` utilise une **chaîne de if/else par ID** :

```
if requires_deposit        → Miner
else if default_recipe + fuel → RecipeGenerator
else if default_recipe     → Assembler
else if id == "turret"     → TurretCombat
else if id == "storage"    → Storage
else if id == "farm"       → Farm
else if id == "archive"    → Archive
else if id == "compactor"  → Compactor
else if id == "pipe"       → FluidPipe
else if id == "pump"       → Pump
else                        → RIEN (bug #1)
```

**Bug #1** : Les bâtiments `has_recipes = true` sans `default_recipe` (workbench, anvil) tombent dans le `else` → jamais de `Assembler`.

**Bug #2** : Si on ajoute un nouveau bâtiment dans le TOML, il faut aussi ajouter une branche dans `build.rs` → c'est l'inverse du data-driven.

## Fix : une approche par propriétés

Au lieu de brancher par ID, on branche par **propriétés du `BuildingDef`**. Chaque composant ECS est ajouté si et seulement si le TOML a le champ correspondant.

### Nouvelle logique (après les blocs spéciaux)

```rust
// src/economy/placement/build.rs

// ── Blocs spéciaux (conservés tels quels) ──

if def.requires_deposit {
    // Trouve le dépôt, le consomme, crée le Miner + Assembler avec mine_recipe
    // (bloc existant, lignes 428-481)
}

if def.id == "turret" || def.id == "turret_ii" {
    // TurretCombat (existant, lignes 565-579)
}

if def.id == BUILDING_STORAGE {
    // Storage marker (existant, lignes 580-582)
}

if def.id == BUILDING_FARM {
    // Farm component (existant, lignes 583-596)
}

if def.id == BUILDING_ARCHIVE {
    // Archive marker (existant, lignes 597-599)
}

// ── Bloc générique data-driven ──

// Tous les autres bâtiments : on ajoute les composants selon les propriétés TOML
let mut e = commands.spawn((base, inv, Level(def.level)));

// Assembler (recettes)
if def.has_recipes || def.default_recipe.is_some() || def.production_interval.is_some() {
    let interval = def.production_interval.unwrap_or(2.0);
    let recipe_id = def.default_recipe.clone().unwrap_or_else(|| {
        // Fallback pour les bâtiments has_recipes sans default_recipe (workbench, anvil, etc.)
        "iron_parts_from_scrap".to_string()
    });
    if def.fuel_burn_interval > 0.0 && def.power_generation > 0.0 {
        // RecipeGenerator : bâtiment hybride qui produit ET génère du courant
        e.insert(RecipeGenerator {
            recipe_id: recipe_id.clone(),
            production_timer: 0.0,
            interval,
            base_output: def.power_generation,
        });
    } else {
        e.insert(Assembler {
            production_timer: 0.0,
            interval,
            recipe_id,
        });
    }
    e.insert(ProductionCounter::default());
    e.insert(DiscoveredRecipes::default());
}

// Power
if def.power_consumption > 0.0 {
    e.insert(PowerConsumer { draw: def.power_consumption, satisfied: false });
}
if def.power_generation > 0.0 {
    e.insert(PowerProducer { output: def.power_generation });
}
if def.fuel_burn_interval > 0.0 {
    e.insert(BurnerGenerator {
        burn_timer: 0.0,
        interval: def.fuel_burn_interval,
    });
}
if def.power_pole_range > 0.0 {
    e.insert(PowerPole { range: def.power_pole_range });
}

// Fluids
if def.fluid_tank_capacity > 0.0 {
    e.insert(FluidTank::new(def.fluid_tank_capacity));
}
if def.id == BUILDING_PIPE {
    e.remove::<FluidTank>();
    e.insert(FluidPipe {
        transfer_rate: def.pipe_transfer_rate,
        direction: Direction::East,
    });
}
if def.id == BUILDING_PUMP {
    e.insert(Pump);
}

// Compactor
if def.compactor_ratio > 0 {
    e.insert(Compactor {
        ratio: def.compactor_ratio,
        timer: 0.0,
        interval: def.compactor_interval,
    });
}
```

### Résumé des règles

| Propriété TOML | Composant ajouté |
|----------------|-----------------|
| `has_recipes = true` ou `default_recipe = "..."` | `Assembler` (ou `RecipeGenerator` si hybride) |
| `power_consumption = N` | `PowerConsumer` |
| `power_generation = N` | `PowerProducer` |
| `fuel_burn_interval = N` | `BurnerGenerator` |
| `power_pole_range = N` | `PowerPole` |
| `fluid_tank_capacity = N` | `FluidTank` |
| `compactor_ratio = N` | `Compactor` |
| `pipe_transfer_rate = N` | `FluidPipe` (uniquement pipe) |
| `requires_deposit = true` | `Miner` (bloc spécial) |

### Avantages

1. **Plus de bug #1** : workbench, anvil reçoivent un `Assembler` automatiquement si `has_recipes = true`
2. **Plus de bug #2** : n'importe quel bâtiment défini dans le TOML reçoit automatiquement les bons composants
3. **Un mod peut ajouter un bâtiment avec `power_consumption = 5`** → le composant `PowerConsumer` est ajouté sans toucher au Rust
4. **Moins de code** : la chaîne de if/else par ID disparaît

### Fichier modifié

`src/economy/placement/build.rs` — réécrire le bloc `else` générique et harmoniser les conditions.

## Prérequis

```diff
- // BuildingDef a déjà tous les champs nécessaires
- // Aucun nouveau champ à ajouter
```
