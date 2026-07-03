# Architecture — Siege Factory

## Principe

ECS (Entity Component System) via Bevy 0.19. Tout le jeu est construit en plugins indépendants, systèmes atomiques, communication par Events.

## Structure des modules (actuelle)

```
src/
├── main.rs                       # Entry point, appelle lib::run()
├── lib.rs                        # App builder, ordre des plugins
├── events.rs                     # Nettoyage entre parties (CleanupPlugin)
│
├── core/                         # Infrastructure base
│   ├── mod.rs
│   ├── game_state.rs             # GameState enum (Menu, Playing, GameOver)
│   ├── schedule.rs               # Tests de cycle de vie
│   ├── input.rs                  # KeyBindings, InputBinding, rebinding
│   ├── main_menu.rs              # Écran titre (Play/Options)
│   ├── settings.rs               # Config graphique/audio/gameplay
│   ├── toast.rs                  # Notifications temporaires
│   └── tooltip.rs                # Infobulles hover
│
├── map/                          # Carte et tuiles
│   ├── mod.rs
│   ├── tile_grid.rs              # Grille 20×15, Tile/TileType/TilePosition
│   ├── components.rs             # Components de rendu map
│   ├── generation.rs             # Génération terrain + ressources
│   └── systems.rs                # Setup carte, rendu, cleanup
│
├── economy/                      # Ressources, bâtiments, menu, placement
│   ├── mod.rs
│   ├── resource.rs               # ResourceId, Inventory, ResourceRegistry
│   ├── building.rs               # BuildingDef, BuildingRegistry, charges données
│   ├── menu.rs                   # MenuDef, MenuState, MenuItems, flat_items_at()
│   ├── build_bar.rs              # UI barre de construction (affichage + interaction)
│   ├── placement.rs              # Placement système (clic → build, ghost, rotation)
│   ├── components.rs             # Components économie (Building, Produces, Belt, etc.)
│   ├── recipe.rs                 # RecipeBank, craft system
│   └── unit_config.rs            # UnitDef, UnitConfig (chargé depuis units.toml)
│
├── enemy/                        # Ennemis et vagues
│   ├── mod.rs
│   ├── wave_state.rs             # Wave definitions, spawn timer, WIN_WAVES
│   ├── ai.rs                     # BFS pathfinding sur grille
│   ├── components.rs             # Enemy, Health, EnemyBundle
│   ├── systems.rs                # Spawn, move, combat, game over UI
│   └── combat.rs                 # Tir automatique des tourelles
│
├── unit/                         # Unités joueur
│   ├── mod.rs                    # Systems spawn/input (menu → SpawnUnitEvent)
│   ├── components.rs             # Soldier, Worker components
│   └── data.rs                   # UnitConfig, stats
│
├── combat/                       # Projectiles et dégâts
│   ├── mod.rs
│   └── projectiles.rs            # Homing projectiles, damage system
│
└── rendering.rs                  # Formes Mesh2d, HP bars, tile highlight, belt items
```

### Modules planifiés (destination Factorio)

| Module | Rôle | Quand |
|---|---|---|
| `save/` | Sauvegarde incrémentale par chunk | Après base solo stable |
| `network/` | Multi déterministe (lockstep) | Après solo stable |
| `player/` | Contrôle, sélection, ordres RTS | Progressivement |
| `ui/` | HUD complet, minimap, infos | Progressivement |

## Communication

- **Logique ↔ Logique** : Events (`BuildOrderEvent`, `SpawnUnitEvent`, `SpawnWaveEvent`)
- **Logique ↔ UI** : Resources (`MenuItems`, `BuildMode`, `DeconstructMode`, `TooltipText`)
- **Logique ↔ Rendu** : Query ECS (séparation naturelle en Bevy)

## Dépendances entre plugins (actuelles)

```
DefaultPlugins
  ├── CorePlugin
  ├── MapPlugin
  ├── EconomyPlugin
  ├── EnemyPlugin
  ├── UnitPlugin
  ├── CombatPlugin
  ├── RenderPlugin
  └── CleanupPlugin
```

## Scalabilité (vers la destination)

### Ce qui scale bien

- **Data-driven** : ajouter un bâtiment/ennemi/recette = juste un TOML, pas de code Rust
- **Events** : découplage total entre UI et logique, nécessaire pour le multi
- **ECS** : composition de components, pas d'héritage
- **String IDs** : pas d'enums Rust figés pour les types de building/ressource (prêt pour N ressources)

### Ce qui changera

| Aujourd'hui | Demain |
|---|---|
| Grille 20×15 fixe (`TileGrid::new`) | Chunks 32×32 chargés/déchargés |
| BFS plein grille | Pathfinding hiérarchique (chunk A* + BFS local) |
| 3 resources en enum (`ResourceId`) | N ressources via IDs dynamiques (String) |
| Rendu Mesh2d | Sprites/atlas avec LOD |

## Règle d'or

Un système ECS ne fait **qu'une seule chose**. Si un système modifie plus d'un type de composant non-trivial, il doit être divisé.
