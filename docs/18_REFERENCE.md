# Reference — Siege Factory

## GameState

```rust
enum GameState { Menu, Playing, GameOver }
```

Transitions :

| From | To | Condition |
|---|---|---|
| Menu | Playing | Menu click (Play) |
| Playing | GameOver | Escape (ou HQ détruit) |
| GameOver | Playing | R |
| GameOver | Menu | Escape |

## Resources ECS

| Resource | Rôle |
|---|---|
| `BuildingRegistry` | Définitions des buildings |
| `ResourceRegistry` | Définitions des ressources |
| `RecipeBank` | Toutes les recettes de craft |
| `EnemyRegistry` | Définitions des ennemis |
| `UnitConfig` | Définitions des unités joueur |
| `MapConfig` | Configuration de la carte |
| `WaveConfig` | Configuration des vagues |
| `MenuDef` | Arbre du menu de construction |
| `MenuState` | Position courante dans le menu |
| `MenuItems` | Items plats affichés dans la barre |
| `BuildMode` | Mode placement actif |
| `DeconstructMode` | Mode démolition |
| `TooltipText` | Texte d'infobulle courante |
| `KeyBindings` | Touches configurables |
| `Settings` | Configuration utilisateur |

## Events

| Event | Producteur | Consommateur |
|---|---|---|
| `BuildOrderEvent` | UI / menu | Placement |
| `SpawnUnitEvent` | UI / menu | Unit spawn |
| `SpawnWaveEvent` | Wave timer | Enemy spawn |
| `GameOverEvent` | Combat | UI |

## Components ECS

| Component | Ajouté à | Rôle |
|---|---|---|
| `TilePosition { x, y }` | Buildings, units | Position sur la grille |
| `Building { kind, name }` | Buildings | Type et nom |
| `Health { current, max }` | Buildings, enemies, units | Points de vie |
| `Inventory { resources }` | Buildings, HQ | Stockage local |
| `Enemy` | Ennemis | Marqueur |
| `EnemyKind(String)` | Ennemis | Type d'ennemi |
| `Produces { timer, interval }` | Miner, Assembler | Production |
| `Belt { direction }` | Belt | Ceinture |
| `BeltItem` | Items sur belts | Item en transit |
| `Soldier` | Soldier | Marqueur unité |
| `Worker` | Worker | Marqueur unité |
| `TurretCombat { timer, interval }` | Turret | Tir automatique |
| `OnDeposit` | Miner | Sur un gisement |
| `Splitter` | Splitter | Marqueur routeur |
| `Sorter` | Sorter | Marqueur filtre |
