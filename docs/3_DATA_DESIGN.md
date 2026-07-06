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
    ├── ResourceRegistry     → ResourceId + ResourceDef
    ├── BuildingRegistry     → Vec<BuildingDef> (String IDs)
    ├── RecipeRegistry       → HashMap<String, RecipeDef>
    ├── DiscoveryRegistry    → Vec<DiscoveryDef> (seuils + récompenses)
    ├── EnemyRegistry        → Vec<EnemyDef>
    ├── WaveConfig           → WaveDefinition
    ├── MapConfig            → Taille grille, dépôts, HQ
    ├── UnitConfig           → HashMap<String, UnitDef>
    ├── MenuDef              → Arbre de menu (récursif)
    └── KeyBindings          → HashMap<InputBinding, KeyCode>
    │
    ▼
Systèmes de jeu (lisent les registres, ne contiennent pas de data)
```

## Identifiants

- **ResourceId** : `String` (ex: `"iron_plate"`, `"circuit"`, `"motor"`)
- **Building kind** : `String` (ex: `"miner"`, `"assembler"`, `"archive"`)
- **Enemy kind** : `String`
- **Unit kind** : `String`
- **Recipe ID** : `String` (ex: `"steel"`, `"motor"`, `"gear"`)

## Structure des fichiers data/

```
data/
├── resources.toml      # Définitions des ressources (nom, couleur, max_stack)
├── buildings.toml      # Définitions des bâtiments (coûts, HP, stats, recettes, dépôt)
├── recipes.toml        # Recettes de craft (input → output, temps, catégorie)
├── discoveries.toml    # [NOUVEAU] Seuils de découverte par bâtiment
├── enemies.toml        # Types d'ennemis (HP, vitesse, dégâts)
├── waves.toml          # Définitions des vagues (composition, timing)
├── units.toml          # Unités joueur (soldier, worker, cultivator)
├── crops.toml          # Définitions des cultures (wheat, wood)
├── menu.toml           # Arbre du menu de construction (catégories, sous-menus)
├── map_config.toml     # Configuration de la carte (taille, dépôts, HQ, seed)
├── keybindings.toml    # Touches par défaut
└── main_menu.toml      # Configuration du menu principal (boutons, thème)
```

## Types évolutifs

Tous les IDs sont des `String` chargées depuis TOML. Aucune enum Rust figée — ajouter une ressource ou un bâtiment ne nécessite **aucune modification du code** (juste du TOML + éventuellement une entrée dans `menu.toml`).

## Avantages

1. **Ajouter un building** = ajouter une section dans `buildings.toml` + éventuellement l'ajouter dans `menu.toml`
2. **Ajouter une découverte** = ajouter une entrée dans `discoveries.toml`
3. **Équilibrer le jeu** = modifier un fichier TOML, pas le code
4. **Modding futur** = les moddeurs peuvent ajouter leurs propres fichiers TOML
5. **Tests** = injecter des registres mockés sans fichiers réels
