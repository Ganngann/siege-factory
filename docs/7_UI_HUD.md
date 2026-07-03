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
│ [←1 Retour] ◀ [⛏️Miner] [🏭Assembler] ... ▶      │ ← Bottom bar (menu)
└──────────────────────────────────────────────────┘
```

## Éléments

### Top bar (HUD)
- Ressources courantes (icône + quantité)
- Vague actuelle / totale
- HP du HQ (barre de vie)

### Zone de jeu
- Grille tuilée 20×15
- Buildings, ennemis, unités affichés par dessus (Mesh2d formes)
- Highlight tuile survolée
- Items sur belts (billes colorées)

### Bottom bar (menu construction)
- **Barre de construction** : menu arborescent data-driven
  - Catégories racines affichées comme boutons
  - Navigation par clic ou touches 2-0
  - 🔙 Retour (touche 1 ou Backspace)
  - ◀ ▶ scroll si >9 items
  - Tooltip au survol avec infos complètes
- **Breadcrumb** : "Production > Tris"
- Icônes emoji pour chaque bâtiment/unité/action

### Main menu (écran titre)
- Boutons Jouer / Options / Quitter
- Configuré via `data/main_menu.toml`

## Events UI → Logique

```rust
BuildOrderEvent { kind: String, pos: TilePosition }
SpawnUnitEvent { kind: String, pos: TilePosition }
```

## Resources partagées UI/Logique

```rust
struct MenuState { stack: Vec<usize>, scroll: usize }
struct MenuItems { items: Vec<FlatItem>, has_back: bool, ... }
struct BuildMode(Option<String>);
struct DeconstructMode(bool);
struct TooltipText(Option<String>);
```

## Règles

- L'UI ne modifie **jamais** directement un component ECS de jeu (Inventory, HP, etc.)
- L'UI lit des Resources publiques, envoie des Events
- Les systèmes de logique traitent les Events, mettent à jour les Resources si nécessaire
- Le rendu des formes est géré par Bevy (Query standard), pas par l'UI
