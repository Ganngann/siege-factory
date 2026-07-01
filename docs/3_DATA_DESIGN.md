# Data Design — Siege Factory

## Principe

Toutes les définitions de données du jeu (ressources, buildings, ennemis, recettes, vagues) sont dans des fichiers `data/*.toml` chargés au démarrage. Le code ne contient **aucune valeur en dur** — uniquement les traits et registres qui les manipulent.

## Architecture

```
data/*.toml
    │
    ▼
AssetLoader (système startup)
    │
    ▼
Registries (Resources ECS)
    │
    ├── ResourceRegistry   → HashMap<ResourceId, ResourceDef>
    ├── BuildingRegistry   → HashMap<BuildingKind, BuildingDef>
    ├── RecipeBank         → Vec<Recipe>
    ├── EnemyRegistry      → HashMap<EnemyKind, EnemyDef>
    └── WaveRegistry       → Vec<WaveDefinition>
    │
    ▼
Systèmes de jeu (lisent les registres, ne contiennent pas de data)
```

## Trait ResourceType

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceId {
    Ore,
    Ammo,
    Energy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceDef {
    id: ResourceId,
    name: String,
    max_stack: u32,
    icon: String, // chemin sprite
}

#[derive(Resource)]
struct ResourceRegistry {
    resources: HashMap<ResourceId, ResourceDef>,
}
```

## Trait BuildingKind

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BuildingKind {
    HQ,
    Miner,
    Assembler,
    Belt,
    Turret,
    Wall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BuildingDef {
    kind: BuildingKind,
    name: String,
    cost: HashMap<ResourceId, u32>,
    hp: u32,
    tile_size: (u32, u32),
    production: Option<ProductionDef>,
}

#[derive(Resource)]
struct BuildingRegistry {
    buildings: HashMap<BuildingKind, BuildingDef>,
}
```

## Recettes

Définies dans `data/recipes.toml`. Une recette transforme des inputs en outputs avec un temps donné.

```toml
[recipes.ammo]
input = { ore = 3 }
output = { ammo = 1 }
time_sec = 2.0
```

## Structure des fichiers data/

```
data/
├── resources.toml      # Définitions des ressources
├── buildings.toml      # Définitions des buildings (coûts, HP, stats)
├── recipes.toml        # Recettes de craft
├── enemies.toml        # Types d'ennemis (HP, vitesse, dégâts)
├── waves.toml          # Définitions des vagues
└── upgrades.toml       # Arbre de technologies (plus tard)
```

## Avantages

1. **Ajouter une ressource** = ajouter une ligne dans `resources.toml` + un variant dans `ResourceId`
2. **Ajouter un building** = ajouter une section dans `buildings.toml` + un plugin de 50 lignes max
3. **Équilibrer le jeu** = modifier un fichier TOML, pas le code
4. **Modding futur** = les moddeurs peuvent ajouter leurs propres fichiers TOML
5. **Tests** = injecter des registres mockés sans fichiers réels
