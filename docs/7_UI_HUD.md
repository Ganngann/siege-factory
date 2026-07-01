# UI / HUD — Siege Factory

## Principes

- UI découplée de la logique : les systèmes UI lisent des Resources, ne modifient pas l'état de jeu directement.
- Les actions utilisateur (clic sur bouton build) envoient des Events que les systèmes de logique traitent.
- Pas d'ECSs dans l'UI (sauf pour les composants UI Bevy).

## Structure de l'interface

```
┌──────────────────────────────────────────────────┐
│ [Ressources: Ore 120 | Ammo 45]   [Vague 3/10] │ ← Top bar
├──────────────────────────────────────────────────┤
│                                                   │
│                                                   │
│               Carte de jeu (tuiles)               │ ← Zone principale
│                                                   │
│                                                   │
├──────────────────────────────────────────────────┤
│ [Menu construction]   [Info building]  [Minimap] │ ← Bottom panel
└──────────────────────────────────────────────────┘
```

## Éléments

### Top bar (HUD)
- Ressources courantes (icône + quantité)
- Vague actuelle / totale
- Timer avant prochaine vague
- HP du HQ (barre de vie)

### Zone de jeu
- Grille tuilée 2D
- Buildings, ennemis, unités affichés par dessus
- Highlight tuile survolée
- Selection box (drag)
- Fog of war (plus tard)

### Bottom panel
- **Build menu** : grille d'icônes des buildings disponibles. Coût affiché. Grisé si pas assez de ressources.
- **Info panel** : quand un building/ennemi est sélectionné, affiche ses stats (HP, production, etc.)
- **Minimap** : vue réduite de la carte (plus tard)

## Events UI → Logique

```rust
// Émis par l'UI, traités par les systèmes de logique
BuildOrderEvent { kind: BuildingKind, pos: TilePosition }
CancelBuildEvent { entity: Entity }
SelectEntityEvent { entity: Entity }
DeselectAllEvent
OpenBuildMenuEvent
CloseBuildMenuEvent
```

## Resources partagées UI/Logique

```rust
#[derive(Resource, Default)]
struct SelectedEntity(Option<Entity>);

#[derive(Resource, Default)]
struct HoveredTile(Option<TilePosition>);

#[derive(Resource, Default)]
struct BuildMode(Option<BuildingKind>);
```

## Règles

- L'UI ne modifie **jamais** directement un component ECS de jeu (Invertory, HP, etc.)
- L'UI lit des Resources publiques, envoie des Events
- Les systèmes de logique traitent les Events, mettent à jour les Resources si nécessaire
- Le rendu des sprites est géré par Bevy (Query standard), pas par l'UI
