# Architecture — Siege Factory

## Principe

ECS (Entity Component System) via Bevy 0.14. Tout le jeu est construit en plugins indépendants, systèmes atomiques, communication par Events.

## Structure des modules

```
src/
├── main.rs                       # Entry point, appelle lib::run()
├── lib.rs                        # App builder, ordre des plugins
│
├── core/                         # Infrastructure base
│   ├── mod.rs
│   ├── game_state.rs             # GameState enum (Loading, Playing, GameOver)
│   ├── schedule.rs               # SystemSets, ordre d'exécution
│   ├── config.rs                 # Settings, constants chargées
│   ├── asset_loader.rs           # Charge data/*.toml en registres
│   └── debug.rs                  # Overlay FPS, inspecteur (dev)
│
├── map/                          # Carte et tuiles
│   ├── mod.rs
│   ├── tile_grid.rs              # Grille de tuiles
│   ├── components.rs             # TilePosition, TileType
│   ├── generation.rs             # Génération terrain + ressources
│   └── systems.rs                # Setup carte, rendu
│
├── economy/                      # Ressources, inventaires, recettes
│   ├── mod.rs
│   ├── registry.rs               # ResourceRegistry, ResourceType trait
│   ├── inventory.rs              # Inventory component
│   ├── recipe.rs                 # RecipeBank, craft system
│   └── systems.rs                # Production, transport
│
├── buildings/                    # Construction et gestion
│   ├── mod.rs
│   ├── registry.rs               # BuildingRegistry, BuildingDef
│   ├── placement.rs              # Placement système (clic → build)
│   ├── miner.rs                  # Plugin mine
│   ├── assembler.rs              # Plugin assembleur
│   ├── belt.rs                   # Plugin ceinture
│   ├── turret.rs                 # Plugin tourelle
│   ├── wall.rs                   # Plugin mur
│   └── hq.rs                     # Quartier général
│
├── enemies/                      # Ennemis et vagues
│   ├── mod.rs
│   ├── registry.rs               # EnemyRegistry
│   ├── wave_spawner.rs           # Wave definitions, spawn timer
│   ├── ai.rs                     # Pathfinding A*
│   └── components.rs             # EnemyBundle
│
├── combat/                       # Combat et dégâts
│   ├── mod.rs
│   ├── damage.rs                 # Damage system, HP
│   ├── projectiles.rs            # Tirs, collisions
│   └── systems.rs
│
├── player/                       # Contrôle joueur
│   ├── mod.rs
│   ├── input.rs                  # Clic, sélection, ordres
│   ├── camera.rs                 # Scroll, zoom
│   └── selection.rs              # Selection box, unités sélectionnées
│
├── ui/                           # Interface utilisateur
│   ├── mod.rs
│   ├── hud.rs                    # Barre ressources, infos building
│   ├── build_menu.rs             # Palette de construction
│   ├── tooltip.rs                # Infobulles
│   └── game_over.rs              # Écran fin de partie
│
├── save/                         # Sauvegarde
│   ├── mod.rs
│   ├── serializer.rs             # Sérialisation ECS → binary/JSON
│   └── systems.rs                # Auto-save, load
│
└── network/                      # Multi (plus tard)
    ├── mod.rs
    ├── p2p.rs                    # Connexion P2P
    ├── sync.rs                   # Synchronisation d'état
    └── anti_cheat.rs             # Hash vérification
```

## SystemSets — ordre d'exécution

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameStep {
    PreUpdate,         // Input, camera, sélection
    Placement,         // Construction placements
    Production,        // Mines, assembleurs tournent
    Transport,         // Ceintures bougent les items
    SpawnEnemies,      // Vagues apparaissent
    AI,               // Ennemis pathfind
    Combat,           // Tourelles tirent, dégâts
    Cleanup,          // Entités mortes supprimées
    PostUpdate,       // UI, rendu, overlay
}
```

## Communication

- **Logique ↔ Logique** : Events (ex: `SpawnWaveEvent`, `BuildingPlacedEvent`)
- **Logique ↔ UI** : Resources (ex: `Resource<SelectedEntity>`, `Resource<HoveredTile>`)
- **Logique ↔ Rendu** : Query ECS (séparation naturelle en Bevy)

## Dépendances entre plugins

```
CorePlugin (toujours en premier)
  ├── MapPlugin
  ├── EconomyPlugin
  │     └── BuildingPlugin
  │           ├── EnemyPlugin
  │           └── CombatPlugin
  ├── PlayerPlugin
  └── UIPlugin (toujours en dernier)
      └── SavePlugin
NetworkPlugin (optionnel, activé plus tard)
```

## Règle d'or

Un système ECS ne fait **qu'une seule chose**. Si un système modifie plus d'un type de composant non-trivial, il doit être divisé.
