# Data-Driven Refactor — Plan détaillé

> Document provisoire. Tous les changements listés ici doivent être réalisés
> pour qu'ajouter un building/unité dans un TOML ne nécessite **aucune ligne de Rust**.

---

## Problème

Actuellement, `BuildKind` (enum à 5 variants) et `UnitKind` (enum à 2 variants)
sont hardcodés dans les `match` de placement, ghost preview, production, combat,
et UI. Ajouter un type dans `data/*.toml` ne le fait pas apparaître dans le jeu.

---

## Principe

- **Étape 1** : `String` remplace tous les enums de type
- **Étape 2** : Les big `match` deviennent des lectures génériques des registres
- **Étape 3** : Les comportements (produire, combattre, ceinture) sont déclenchés
  par la présence de composants génériques plutôt que par des types spécifiques
- **Étape 4** : L'UI itère les registres dynamiquement

---

## Phasage

### Phase 1 — Data + Registres (prérequis)

Les TOML et leurs parseurs Rust sont mis à jour en premier.

#### 1.1 `data/buildings.toml`

Ajouter `visual` et `requires_deposit` à chaque building (sauf HQ) :

```toml
[buildings.miner]
name = "Miner"
cost = { ore = 10 }
hp = 100
tile_size = { w = 1, h = 1 }
color = "#993300"
visual = "square"
requires_deposit = true
production = { resource = "ore", interval_sec = 2.0 }

[buildings.assembler]
name = "Assembler"
cost = { ore = 15 }
hp = 80
tile_size = { w = 1, h = 1 }
color = "#4D99CC"
visual = "diamond"

[buildings.belt]
name = "Belt"
cost = { ore = 3 }
hp = 20
tile_size = { w = 1, h = 1 }
color = "#808080"
visual = "rectangle"
slots = 4
speed = 2.0

[buildings.wall]
name = "Wall"
cost = { ore = 5 }
hp = 300
tile_size = { w = 1, h = 1 }
color = "#4D4D4D"
visual = "rectangle"

[buildings.turret]
name = "Turret"
cost = { ore = 20, ammo = 5 }
hp = 120
tile_size = { w = 1, h = 1 }
color = "#E63333"
visual = "triangle"
combat = { damage = 5, range = 4.0, fire_rate_sec = 1.0 }
```

Note : le champ `slots` / `speed` de belt est pour l'instant spécifique (pas de
générique `belt = {}` dans le TOML). On garde comme ça pour l'instant — un
building belt se reconnaît à la présence du champ `slots`.

#### 1.2 `data/units.toml`

Ajouter `visual` :

```toml
[soldier]
name = "Soldier"
cost = { ore = 10 }
hp = 30
damage = 8
range_tiles = 3.0
fire_rate_sec = 1.0
color = "#33CC4D"
visual = "pentagon"

[worker]
name = "Worker"
cost = { ore = 5 }
hp = 15
speed = 80.0
mine_interval_sec = 3.0
color = "#E6CC33"
visual = "circle"
```

#### 1.3 `economy/building.rs`

Ajouter les champs `visual` et `requires_deposit` à `BuildingDef` :

```rust
pub struct BuildingDef {
    pub id: String,
    pub name: String,
    pub cost: Vec<BuildingCost>,
    pub hp: u32,
    pub tile_size: (u32, u32),
    pub color: Color,
    pub visual: String,              // ← nouveau
    pub requires_deposit: bool,      // ← nouveau
    pub combat: Option<CombatStats>,
    pub belt: Option<BeltProperties>,
    pub production: Option<ProductionDef>,  // ← nouveau
}
```

Ajouter `ProductionDef` :

```rust
#[derive(Debug, Clone)]
pub struct ProductionDef {
    pub resource: ResourceId,
    pub interval_sec: f32,
}
```

Parser les nouveaux champs dans `BuildingRegistry::load()` :

```rust
let visual = entry.visual.unwrap_or_else(|| "square".to_string());
let requires_deposit = entry.requires_deposit;
let production = entry.production.map(|p| ProductionDef {
    resource: ResourceId::from_str(&p.resource).unwrap_or(ResourceId::Ore),
    interval_sec: p.interval_sec,
});
```

Mettre à jour le TOML entry parser pour gérer les nouveaux champs :

```rust
#[derive(Deserialize)]
struct BuildingEntry {
    name: String,
    #[serde(default)]
    cost: HashMap<String, u32>,
    hp: u32,
    tile_size: TileSize,
    color: Option<String>,
    #[serde(default)]
    visual: Option<String>,
    #[serde(default)]
    requires_deposit: bool,
    #[serde(default)]
    production: Option<ProductionEntry>,
    #[serde(default)]
    combat: Option<CombatEntry>,
    #[serde(default)]
    belt: Option<BeltEntry>,
}

#[derive(Deserialize)]
struct ProductionEntry {
    resource: String,
    interval_sec: f32,
}
```

#### 1.4 `economy/unit_config.rs`

Remplacer `SoldierDef` / `WorkerDef` par une `HashMap<String, UnitDef>` unifiée.

```rust
#[derive(Debug, Clone)]
pub struct UnitDef {
    pub id: String,
    pub name: String,
    pub cost: Vec<UnitCost>,
    pub hp: u32,
    pub color: Color,
    pub visual: String,
    pub kind: String,           // "combat" | "harvester"
    pub damage: Option<u32>,
    pub range_tiles: Option<f32>,
    pub fire_rate_sec: Option<f32>,
    pub speed: Option<f32>,
    pub mine_interval_sec: Option<f32>,
}

#[derive(Debug, Clone, Resource)]
pub struct UnitConfig {
    pub units: HashMap<String, UnitDef>,
}
```

Parser génériquement depuis TOML avec `#[serde(flatten)]` ou des champs optionnels.

#### 1.5 `rendering.rs`

Ajouter `get_visual(&self, visual: &str) -> Handle<Mesh>` :

```rust
impl ShapeCache {
    pub fn get_visual(&self, visual: &str) -> Handle<Mesh> {
        match visual {
            "square" => self.square.clone(),
            "diamond" => self.diamond.clone(),
            "triangle" => self.triangle.clone(),
            "rectangle" => self.rectangle.clone(),
            "pentagon" => self.pentagon.clone(),
            "circle" => self.circle.clone(),
            _ => self.square.clone(), // fallback
        }
    }
}
```

Plus tard, quand les sprites arriveront, cette fonction pourra d'abord
chercher une texture sprite, puis tomber sur le fallback shape.

---

### Phase 2 — Components + Events (coeur ECS)

#### 2.1 `economy/components.rs`

Supprimer `BuildKind` enum. Remplacer `BuildMode` et `SetBuildModeEvent` :

```rust
#[derive(Resource, Default)]
pub struct BuildMode(pub Option<String>);

#[derive(Event)]
pub struct SetBuildModeEvent(pub Option<String>);
```

Ajouter les composants génériques de comportement :

```rust
#[derive(Component)]
pub struct Produces {
    pub resource: ResourceId,
    pub interval: f32,
    pub timer: f32,
}

#[derive(Component, Clone)]
pub struct TurretCombat {
    pub damage: u32,
    pub range_sq: f32,
    pub fire_interval: f32,
    pub timer: f32,
}

#[derive(Component)]
pub struct Unit;
```

Modifier `Building` component pour stocker `kind` (ID) au lieu de `name` :

```rust
#[derive(Component)]
pub struct Building {
    pub kind: String,   // ← "miner", "assembler", etc.
}
```

#### 2.2 `events.rs`

Ajouter `BuildOrderEvent` pour respecter le principe "UI → Event → Logique" :

```rust
#[derive(Event)]
pub struct BuildOrderEvent {
    pub kind: String,
    pub pos: TilePosition,
}
```

#### 2.3 Supprimer les dépendances à `BuildKind`

Chercher `use ... BuildKind` dans tous les fichiers et remplacer par `String`.
Chercher les `_ => continue` dans les `match` sur `BuildKind`.

#### 2.4 Migrer `cleanup_game_entities` vers le tag `Unit`

**`enemy/systems.rs`** — remplacer `Or<(With<Soldier>, With<Worker>)>` par
`With<Unit>` :

```rust
// avant
fn cleanup_game_entities(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<TilePosition>)>,
    soldiers_and_workers: Query<Entity, Or<(With<Soldier>, With<Worker>)>>,
) { ... }

// après
fn cleanup_game_entities(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<TilePosition>)>,
    units: Query<Entity, With<Unit>>,
) { ... }
```

Ce changement peut être fait indépendamment du reste : le tag `Unit` est ajouté
en Phase 2.1, le cleanup est mis à jour ici, et le spawn des unités attachera
le tag en Phase 6.

---

### Phase 3 — Placement (le plus gros morceau)

#### 3.1 `economy/placement.rs`

**`build_mode_input`** : remplacer le key_map fixe par une itération dynamique
sur le registry :

```rust
let buildings: Vec<_> = registry.buildings.iter()
    .filter(|b| b.id != "hq").collect();
for (i, key) in [Digit1, Digit2, Digit3, Digit4, Digit5].iter().enumerate() {
    if keys.just_pressed(*key) {
        if let Some(def) = buildings.get(i) {
            build_mode.0 = match &build_mode.0 {
                Some(id) if id == &def.id => None,
                _ => Some(def.id.clone()),
            };
        }
    }
}
```

**`update_build_preview`** : remplacer le `match kind` géant par une lecture
générique :

```rust
let Some(kind) = &build_mode.0 else { ... };
let Some(def) = registry.get(kind) else { ... };

let valid = if def.requires_deposit {
    // vérifier que la tuile a un dépôt
    deposits.iter().any(|pos| pos.x == tx && pos.y == ty)
        && !miners_deposits.iter().any(|pos| pos.x == tx && pos.y == ty)
} else {
    tile_is_free(tx, ty, &buildings)
};

let mesh = shapes.get_visual(&def.visual);
// spawn ghost avec cette mesh
```

**`handle_build_click`** : ne plus spawn directement. À la place, envoyer
`BuildOrderEvent` :

```rust
pub fn handle_build_click(
    ...
    mut events: EventWriter<BuildOrderEvent>,
) {
    // ... calcul de la tuile, validation BuildMode
    // ... validation coût, dépôt
    events.send(BuildOrderEvent { kind: build_mode.0.clone().unwrap(), pos: TilePosition { x: tx, y: ty } });
}
```

**NOUVEAU `handle_build_order`** : recevoir l'event, spawner le building :

```rust
pub fn handle_build_order(
    mut commands: Commands,
    mut events: EventReader<BuildOrderEvent>,
    registry: Res<BuildingRegistry>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cfg: Res<MapConfig>,
    deposit_events: EventWriter<DespawnDeposit>,
    hq_query: Query<&mut Inventory, With<HQ>>,
) {
    for ev in events.read() {
        let def = match registry.get(&ev.kind) { ... };
        // déduire le coût du HQ
        // spawner génériquement :
        let mut bundle = (
            Building { kind: def.id.clone() },
            Inventory::new(),
            ColorMesh2dBundle {
                mesh: Mesh2dHandle(shapes.get_visual(&def.visual)),
                material: material_from_color(&mut materials, def.color),
                transform: ...,
                ..default()
            },
            TilePosition { x: ev.pos.x, y: ev.pos.y },
        );
        if let Some(prod) = &def.production {
            commands.spawn(bundle).insert(Produces {
                resource: prod.resource,
                interval: prod.interval_sec,
                timer: 0.0,
            });
        } else if let Some(combat) = &def.combat {
            commands.spawn(bundle).insert(TurretCombat {
                damage: combat.damage,
                range_sq: combat.range,
                fire_interval: combat.fire_rate_sec,
                timer: 0.0,
            });
        } else if let Some(belt) = &def.belt {
            // spawn avec BeltSlots (spécifique)
        } else {
            commands.spawn(bundle);
        }
    }
}
```

Note : le `if` chaîné ci-dessus est un placeholder. La version finale devrait
être plus composable (ex: `bundle.with(Produces { ... })`).

---

### Phase 4 — Production

#### 4.1 `economy/production.rs`

**`production_tick`** : remplacer `Query<&mut Miner>` par `Query<&mut Produces>` :

```rust
pub fn production_tick(
    time: Res<Time>,
    mut producers: Query<(&mut Produces, &TilePosition)>,
    mut events: EventWriter<SpawnBeltItemEvent>,
) {
    for (mut prod, pos) in &mut producers {
        prod.timer += time.delta_seconds();
        while prod.timer >= prod.interval {
            prod.timer -= prod.interval;
            events.send(SpawnBeltItemEvent {
                source_tile: *pos,
                resource: prod.resource,
            });
        }
    }
}
```

`assembler_tick` reste spécifique (logique de consommation de belts). On garde
un component `RecipeConsumer` ou un tag pour le query :

```rust
#[derive(Component)]
pub struct RecipeConsumer;
```

L'Assembler sera spawné avec `RecipeConsumer` dans `handle_build_order`
(via une détection `def.id == "assembler"` — c'est le seul building avec
recette, acceptable pour l'instant).

---

### Phase 5 — Combat

#### 5.1 `enemy/combat.rs`

**`turret_shoot`** : remplacer `Query<(&Transform, &mut Turret)>` par
`Query<(&Transform, &mut TurretCombat)>` :

```rust
pub fn turret_shoot(
    mut commands: Commands,
    mut turrets: Query<(&Transform, &mut TurretCombat)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (pos, mut combat) in turrets.iter_mut() {
        combat.timer += time.delta_seconds();
        if combat.timer < combat.fire_interval {
            continue;
        }
        // trouver target dans combat.range_sq
        // tirer projectile avec combat.damage
        combat.timer -= combat.fire_interval;
    }
}
```

---

### Phase 6 — Unités

#### 6.1 `unit/mod.rs`

**Supprimer** `UnitKind` enum. **Remplacer** `SpawnUnitEvent(UnitKind)` par
`SpawnUnitEvent(String)`.

**`spawn_unit_input`** : itérer le `UnitConfig.units` pour les touches clavier
(6, 7) de manière dynamique. Pour les events, chercher dans la HashMap :

```rust
fn spawn_unit_input(..., mut spawn_events: EventReader<SpawnUnitEvent>) {
    // touches 6, 7 → premier/second du registry
    for ev in spawn_events.read() {
        if let Some(def) = unit_cfg.units.get(&ev.0) {
            if inv.get(ResourceId::Ore) < def.cost.iter().find(|c| c.resource == ResourceId::Ore).map(|c| c.amount).unwrap_or(0) {
                continue;
            }
            // déduire coût
            // spawner génériquement
            if def.kind == "combat" {
                // spawn CombatUnit + CombatStats
            } else if def.kind == "harvester" {
                // spawn Worker
            }
        }
    }
}
```

**`soldier_auto_attack`** : remplacer `Query<(&Transform, &mut Soldier)>` par
`Query<(&Transform, &mut CombatTimer, &CombatStats)>` (à créer, similaire à
ce qu'on a pour les tourelles mais pour les unités mobiles).

**`worker_harvest`** : garder la query `With<Worker>` puisque le comportement
de récolte est trop spécifique pour être générique.

Nettoyage : utiliser le tag `Unit` déjà ajouté en Phase 2.1 pour `cleanup_game_entities` :

```rust
#[derive(Component)]
pub struct Unit;
```

Toutes les unités spawnent avec ce tag. Le cleanup devient :

```rust
fn cleanup_game_entities(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<TilePosition>)>,
    units: Query<Entity, With<Unit>>,
) {
    for entity in enemies.iter().chain(units.iter()) {
        commands.entity(entity).despawn();
    }
}
```

---

### Phase 7 — UI (build bar)

#### 7.1 `economy/build_bar.rs`

**Supprimer** le `match def.id.as_str()` qui mappe string → `BuildKind`.
Remplacer par l'ID string direct :

```rust
// avant
let kind = match def.id.as_str() {
    "miner" => BuildKind::Miner,
    "assembler" => BuildKind::Assembler,
    _ => continue,  // ← inconnu = ignoré
};
// après
let kind = def.id.clone();  // ← String, pas d'enum
```

**Boutons unités** : itérer `unit_cfg.units` au lieu du tuple hardcodé :

```rust
// avant
for (unit_kind, name, cost) in [
    (UnitKind::Soldier, ...),
    (UnitKind::Worker, ...),
] { ... }
// après
for (id, def) in &unit_cfg.units {
    // créer un bouton avec id, def.name, cost
}
```

**`build_bar_interaction`** : envoyer `SetBuildModeEvent(Some(kind))` où
`kind` est un `String`.

**`update_build_bar`** : remplacer la comparaison `build_mode.0 == Some(*kind)`
par `build_mode.0 == Some(kind)` (string vs string).

---

### Phase 8 — Câblage (mod.rs)

#### 8.1 `economy/mod.rs`

- Ajouter `app.add_event::<BuildOrderEvent>()` (depuis `events.rs` ou depuis
  `placement.rs`)
- Ajouter `placement::handle_build_order` à la Update schedule
- Modifier les imports pour ne plus référencer `BuildKind`
- Vérifier que tous les systèmes sont dans le bon ordre

#### 8.2 `enemy/mod.rs`

- Remplacer les imports de `Turret` par `TurretCombat`
- Nettoyage des références à `Soldier` / `Worker` dans `cleanup_game_entities`

#### 8.3 `lib.rs`

- Si `ui/mod.rs` est créé, ajouter `pub mod ui;`

---

### Phase 9 — Nettoyage

#### 9.1 Fichiers à supprimer

- `economy/systems.rs` (s'il existe encore — tout a été déplacé vers
  `components.rs`, `placement.rs`, `production.rs`, `setup.rs`)

#### 9.2 Fichiers à alléger

- `economy/ui.rs` : déjà nettoyé
- `core/schedule.rs` : le menu viendra après

---

## Ordre d'exécution réalisé

Tout le refactor a été fait en une session. Voici l'état final :

```
Phase 1 (Data + Registres)                   ✅
  ├── 1.1 buildings.toml                     ✅ + visual, requires_deposit
  ├── 1.2 units.toml                         ✅ + visual
  ├── 1.3 building.rs                        ✅ + ProductionDef, visual, requires_deposit
  ├── 1.4 unit_config.rs                     ✅ refactor HashMap dynamique
  └── 1.5 rendering.rs                       ✅ + get_visual()
                                              │
Phase 2 (Components + Events)                ✅
  ├── 2.1 components.rs                      ✅ Produces, TurretCombat, Unit
  ├── 2.2 events.rs                          ✅ + BuildOrderEvent
  └── 2.3 BuildKind → String                 ✅ enum supprimé
                                              │
Phase 3 (Placement)                          ✅ (inline avec Phase 2.3)
  └── 3.1 placement.rs                       ✅ générique (registry-based)
                                              │
Phase 4 (Production)                         ✅
  └── 4.1 production.rs                      ✅ Miner → Produces
                                              │
Phase 5 (Combat)                             ✅
  └── 5.1 combat.rs (enemy/)                 ✅ Turret → TurretCombat
                                              │
Phase 6 (Unités)                             ✅
  └── 6.1 unit/mod.rs                        ✅ UnitKind → String, spawn générique
                                              │
Phase 7 (UI build bar)                       ✅
  └── 7.1 build_bar.rs                       ✅ itération dynamique registry + unit_cfg
                                              │
Phase 8 (Câblage)                            ~
  ├── 8.1 economy/mod.rs                     ✅ imports à jour
  └── 8.2 enemy/mod.rs                       ✅ imports à jour (TurretCombat)
                                              │
Phase 9 (Nettoyage)                           ❌ à faire
  ├── 9.1 Tests à écrire                     
  └── 9.2 Supprimer Turret struct (fait), Miner/Assembler components (encore utilisés)
```

## Ce qui reste (optionnel / cleanup)

- Tests unitaires pour les nouveaux systèmes génériques
- Supprimer les composants `Miner`, `Assembler` si plus utilisés (gardés pour
  compatibilité avec `handle_build_click` et `assembler_tick`)
- `BuildOrderEvent` défini mais pas encore branché dans la schedule (utilisation
  future)
- `parse_hex_color` duplication entre `building.rs` et `unit_config.rs`
```

---

## Tests à écrire

| Fonction/Système | Fichier test | Type |
|---|---|---|
| `BuildingRegistry::load()` | `building.rs` tests | existing |
| `UnitConfig::load()` | `unit_config.rs` tests | existing |
| `ShapeCache::get_visual()` | `rendering.rs` tests | unitaire |
| `handle_build_order` spawning | `placement.rs` tests | intégration |
| `update_build_preview` generic | `placement.rs` tests | intégration |
| `production_tick` with `Produces` | `production.rs` tests | intégration |
| `turret_shoot` with `TurretCombat` | `combat.rs` tests | intégration |
| `BuildMode` toggle | `components.rs` tests | unitaire |
| `BuildOrderEvent` dispatch | `events.rs` tests | intégration |

---

## Fichiers impactés (résumé)

| Fichier | +/- lignes estimé |
|---------|-------------------|
| `data/buildings.toml` | +6 |
| `data/units.toml` | +2 |
| `economy/building.rs` | +30 |
| `economy/unit_config.rs` | ~80 (refactor complet) |
| `rendering.rs` | +15 |
| `economy/components.rs` | ~40 (refactor) |
| `events.rs` | +5 |
| `economy/placement.rs` | ~150 (refactor lourd) |
| `economy/production.rs` | ~30 |
| `enemy/combat.rs` | ~20 |
| `unit/mod.rs` | ~100 |
| `economy/build_bar.rs` | ~30 |
| `economy/mod.rs` | ~10 |
| `enemy/mod.rs` | ~10 |
| Tests (tous fichiers) | +~100 |
| **Total** | **~630 lignes** |
