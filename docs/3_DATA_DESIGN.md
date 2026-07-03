# Data Design — Siege Factory

## Principe

Toutes les définitions de données du jeu (ressources, buildings, ennemis, recettes, vagues, menu, unités) sont dans des fichiers `data/*.toml` chargés au démarrage. Le code ne contient **aucune valeur en dur** — uniquement les traits et registres qui les manipulent.

## Architecture

```
data/*.toml
    │
    ▼
Startup systems (lecture include_str! + parsing)
    │
    ▼
Registries (Resources ECS)
    │
    ├── ResourceRegistry   → ResourceId + ResourceDef
    ├── BuildingRegistry   → Vec<BuildingDef> (String IDs)
    ├── RecipeBank         → Vec<Recipe>
    ├── EnemyRegistry      → Vec<EnemyDef>
    ├── WaveConfig         → WaveDefinition
    ├── MapConfig          → Taille grille, dépôts, HQ
    ├── UnitConfig         → HashMap<String, UnitDef>
    ├── MenuDef            → Arbre de menu (récursif)
    └── KeyBindings        → HashMap<InputBinding, KeyCode>
    │
    ▼
Systèmes de jeu (lisent les registres, ne contiennent pas de data)
```

## Identifiants

- **ResourceId** : enum Rust (Ore, Ammo, Energy) — sera migré vers des ID dynamiques (String) plus tard
- **Building kind** : `String` (ex: `"miner"`, `"assembler"`) — lookup par ID dans `BuildingRegistry::get(id)`
- **Enemy kind** : `String`
- **Unit kind** : `String`

## Structure des fichiers data/

```
data/
├── resources.toml      # Définitions des ressources (nom, icône)
├── buildings.toml      # Définitions des bâtiments (coûts, HP, stats, couleur, icône)
├── recipes.toml        # Recettes de craft (input → output)
├── enemies.toml        # Types d'ennemis (HP, vitesse, dégâts)
├── waves.toml          # Définitions des vagues (composition, timing)
├── units.toml          # Unités joueur (soldier, worker)
├── menu.toml           # Arbre du menu de construction (catégories, sous-menus)
├── map_config.toml     # Configuration de la carte (taille, dépôts, HQ, spawners)
├── keybindings.toml    # Touches par défaut
└── main_menu.toml      # Configuration du menu principal (boutons, thème)
```

## Types évolutifs

Aujourd'hui les IDs de ressources sont une enum Rust à 3 variants (`ResourceId::Ore | Ammo | Energy`).
Quand le jeu aura besoin de N ressources dynamiques, cette enum sera remplacée par un système d'IDs en String chargées depuis TOML.

C'est le seul endroit où le code n'est pas encore totalement data-driven.

## Avantages

1. **Ajouter un building** = ajouter une section dans `buildings.toml` + éventuellement l'ajouter dans `menu.toml`
2. **Équilibrer le jeu** = modifier un fichier TOML, pas le code
3. **Modding futur** = les moddeurs peuvent ajouter leurs propres fichiers TOML
4. **Tests** = injecter des registres mockés sans fichiers réels
