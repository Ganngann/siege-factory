# Architecture — Siege Factory

## Principe

ECS (Entity Component System) via Bevy 0.19. Tout le jeu est construit en plugins indépendants, systèmes atomiques, communication par Events.

## Structure des modules

```
src/
├── core/          # GameState, input, menus, settings, toasts, tooltips
├── map/           # Grid, tiles, generation, components map
├── economy/       # Ressources, inventaires, bâtiments, recettes, menu, placement
├── enemy/         # Ennemis, vagues, pathfinding, combat
├── unit/          # Unités joueur
├── combat/        # Projectiles, dégâts
├── save_load/     # Sauvegarde / chargement
└── rendering/     # Mesh2d, HP bars, highlight, belt items
```

### Modules planifiés

- `network/` : multi déterministe (lockstep)
- `player/` : contrôle, sélection, ordres RTS
- `ui/` : HUD complet, minimap, infos

## Communication

- **Logique ↔ Logique** : Events
- **Logique ↔ UI** : Resources (lecture seule pour UI)
- **Logique ↔ Rendu** : Query ECS (séparation naturelle en Bevy)

## Dépendances entre plugins

```
DefaultPlugins
  ├── CorePlugin
  ├── MapPlugin
  ├── EconomyPlugin
  ├── EnemyPlugin
  ├── UnitPlugin
  ├── CombatPlugin
  ├── SaveLoadPlugin
  ├── RenderPlugin
  └── CleanupPlugin
```

## Scalabilité

### Ce qui scale bien

- **Data-driven** : TOML, pas de code Rust pour ajouter du contenu
- **Events** : découplage UI/logique, nécessaire pour le multi
- **ECS** : composition de components
- **String IDs** : pas d'enums figés

### Ce qui changera

| Aujourd'hui | Demain |
|---|---|
| Grille fixe | Chunks 32×32 chargés/déchargés |
| BFS plein grille | Pathfinding hiérarchique |
| Rendu Mesh2d | Sprites/atlas avec LOD |

## Règle d'or

Un système ECS ne fait qu'une seule chose. Si un système modifie plus d'un type de composant non-trivial, il doit être divisé.
