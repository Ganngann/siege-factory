# Reference — Siege Factory

## GameState

```rust
enum GameState { Menu, Loading, Playing, GameOver }
```

Transitions :

| From | To | Condition |
|---|---|---|
| Menu | Loading | Menu click (Play / Load) |
| Loading | Playing | Sauvegarde chargée ou nouvelle partie prête |
| Playing | GameOver | Escape (menu pause → abandon) |
| GameOver | Playing | R |
| GameOver | Menu | Escape |

## Resources ECS

| Resource | Rôle |
|---|---|
| `BuildingRegistry` | Définitions des buildings (loaded from TOML) |
| `ResourceRegistry` | Définitions des ressources (nom, couleur, stack) |
| `RecipeRegistry` | Toutes les recettes de craft |
| `DiscoveryRegistry` | Définitions des seuils de découverte (loaded from TOML) |
| `GlobalArchive` | HashSet des recettes débloquées définitivement |
| `EnemyRegistry` | Définitions des ennemis |
| `UnitConfig` | Définitions des unités joueur |
| `MapConfig` | Configuration de la carte (taille, seed, dépôts) |
| `WaveConfig` | Configuration des vagues |
| `CropRegistry` | Définitions des cultures |
| `MenuDef` | Arbre du menu de construction |
| `MenuState` | Position courante dans le menu |
| `MenuItems` | Items plats affichés dans la barre |
| `BuildMode` | Mode placement actif |
| `DeconstructMode` | Mode démolition |
| `SpatialRegistry` | Tuile → Entité pour collisions |
| `BuildingPanel` | État du panneau d'inspection |
| `PeacefulMode` | Vagues ON/OFF |
| `ToastQueue` | Notifications temporaires |
| `TooltipText` | Texte d'infobulle courante |
| `KeyBindings` | Touches configurables |
| `Settings` | Configuration utilisateur |

## Events

| Event | Producteur | Consommateur |
|---|---|---|
| `BuildOrderEvent` | UI / menu | Placement |
| `SpawnUnitEvent` | UI / menu | Unit spawn |
| `BeltDragCompleted` | Build click | Placement (drag) |
| `DeconstructAreaEvent` | Deconstruct drag | Destruction zone |
| `SpawnProjectileEvent` | Turret / soldier | Combat rendering |
| `DespawnDeposit(Entity)` | Placement (miner) | Cleanup |
| `DespawnEnemy(Entity)` | Combat | Cleanup |
| `DiscoveryEvent { building, discovery_id }` | check_discoveries | Archive + Toast |

## Components ECS

| Component | Ajouté à | Rôle |
|---|---|---|
| `TilePosition { x, y }` | Buildings, units, deposits | Position sur la grille |
| `Building { kind, name }` | Buildings | Type et nom |
| `OccupiedTiles(Vec<(i32,i32)>)` | Buildings | Empreinte multi-tuiles |
| `Active(bool)` | Buildings | ON/OFF |
| `Inventory { resources, capacity }` | Buildings, HQ | Stockage |
| `Assembler { timer, interval, recipe_id }` | Furnace, Assembler, Miner | Production par recette |
| `Miner` | Miner | Marqueur mineur |
| `TurretCombat { damage, range_sq, timer, ... }` | Turret | Tir automatique |
| `Storage` | Storage | Marqueur stockage |
| `Splitter { counter, outputs }` | Splitter | Routage tourniquet |
| `Sorter { filter, inverted }` | Sorter | Filtrage par type |
| `Farm { crop_index, crop_types }` | Farm | Gestion des cultures |
| `ProductionCounter(u32)` | Furnace, Assembler, Miner, Farm | Compteur de crafts (découverte) |
| `DiscoveredRecipes(Vec<String>)` | Buildings avec découvertes | Recettes débloquées sur ce bâtiment |
| `Archive` | Archive | Marqueur bâtiment d'archivage |
| `Enemy` | Ennemis | Marqueur |
| `Health { current, max }` | Buildings, enemies, units | Points de vie |
| `Unit` | Soldier, Worker | Marqueur unité |
| `HQ` | HQ | Marqueur quartier général |
| `ResourceDeposit { resource, amount }` | Dépôts | Gisement de ressources |
