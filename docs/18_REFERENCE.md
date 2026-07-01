# Reference — Siege Factory

> Auto-généré à partir du code. Cette section sera maintenue à jour par un script après chaque milestone.

## GameState

```rust
enum GameState { Loading, Playing, GameOver }
```

Transitions :

| From | To | Condition |
|---|---|---|
| Loading | Playing | Space |
| Playing | GameOver | Escape |
| GameOver | Playing | R |

## Resources ECS

| Resource | Rôle |
|---|---|
| `GameSeed` | Seed déterministe pour RNG |
| `FrameNumber` | Numéro de frame logique |
| `ResourceRegistry` | Définitions des ressources |
| `BuildingRegistry` | Définitions des buildings |
| `RecipeBank` | Toutes les recettes de craft |
| `EnemyRegistry` | Définitions des ennemis |
| `SelectedEntity` | Entité actuellement sélectionnée |
| `HoveredTile` | Tuile survolée par la souris |
| `BuildMode` | Mode placement actif (si building sélectionné dans le menu) |
| `Settings` | Configuration utilisateur |

## Events

| Event | Producteur | Consommateur |
|---|---|---|
| `BuildOrderEvent` | UI / Input | BuildingPlugin |
| `SpawnWaveEvent` | WaveSpawner | EnemyPlugin |
| `EntityDestroyedEvent` | CombatPlugin | Cleanup, UI |
| `GameOverEvent` | CombatPlugin | UIPlugin |
| `BuildingPlacedEvent` | BuildingPlugin | EconomyPlugin, Audio |
| `ResourceChangedEvent` | EconomyPlugin | UIPlugin (HUD) |

## Components ECS

| Component | Ajouté à | Rôle |
|---|---|---|
| `TilePosition { x, y }` | Tuiles, Buildings | Position sur la grille |
| `Tile { tile_type, occupied }` | Tuiles | Type de terrain |
| `Building { kind, hp }` | Buildings | Type et vie |
| `Inventory { resources }` | Buildings, HQ | Stockage local |
| `Enemy { kind, hp }` | Ennemis | Type et vie |
| `NetworkId(u64)` | Entités persistantes | ID réseau (multi) |
| `OnDeposit(bool)` | Miner | Sur un gisement |

## SystemSets

```rust
Order: PreUpdate → Placement → Production → Transport
       → SpawnEnemies → AI → Combat → Cleanup → PostUpdate
```
