# Spécification TOML UI — Guide du développeur de mods

## 1. Vue d'ensemble

L'interface utilisateur de Siege Factory est **data-driven** : la disposition, les couleurs, les textes et les données affichées sont définis dans des fichiers TOML. Le code Rust ne fournit que les *systèmes réactifs* (populate, update, click).

**Principe :** votre mod fournit des fichiers `panel_*.toml` dans `data/`. Le moteur les charge, les parse et invoque les composants correspondant au champ `type` de chaque élément.

---

## 2. Fichiers panel obligatoires

| Fichier | Déclencheur | Type de rendu |
|---|---|---|
| `panel_hud.toml` | Spawné au `OnEnter(Playing)`, despawné au `OnExit` | HUD (position absolue) |
| `panel_game_over.toml` | Spawné au `OnEnter(GameOver)` | Fullscreen overlay |
| `panel_inventory.toml` | Toggle touche **I** | Fenêtre centrée + overlay |
| `panel_hand_crafting.toml` | Toggle touche **C** | Fenêtre centrée + overlay |
| `panel_build_bar.toml` | Spawné au `OnEnter(Playing)` | Barre pleine largeur en bas |
| `panel_capsule.toml` | Clic sur la capsule | Fenêtre centrée + overlay |

**Panels bâtiments :** tout fichier `panel_{building_id}.toml` dans `data/` est chargé automatiquement quand le joueur clique sur un bâtiment de ce type (ex: `panel_storage.toml`, `panel_production.toml`).

---

## 3. Structure générale d'un panel

### Fenêtre centrée (inventaire, artisanat, inspection)

```toml
title = "Mon Panel"
width = 400
height = 500

[[sections]]
type = "section"
title = "MA SECTION"
elements = [
    { type = "label", text = "Hello", style = "title" },
    { type = "spacer", height = 8 },
    { type = "inventory_grid", cols = 5, rows = 4 },
]
```

### Fullscreen (game over)

```toml
title = "Game Over"
background = "#000000"

[[sections]]
type = "section"
title = ""
elements = [
    { type = "label", text = "GAME OVER", font_size = 48, color = "#ff4d4d" },
    { type = "data_label", key = "game_over.waves", style = "title" },
    { type = "spacer", height = 8 },
    { type = "label", text = "Press R to restart", font_size = 20 },
]
```

### HUD (éléments persistants positionnés)

```toml
title = "HUD"

[[sections]]
type = "hud_text"
data_key = "hud.wave_counter"
font_size = 16
color = "#ffcc44"
position = { top = 10, right = 10 }

[[sections]]
type = "hud_text"
data_key = "hud.fps"
font_size = 12
color = "#00ff00"
position = { top = 40, right = 10 }
```

---

## 4. Catalogue des composants disponibles

### 4.1 Conteneurs

#### `section`
Bloc visuel avec fond et titre optionnel.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `title` | string | `""` | Titre affiché en haut de la section |
| `elements` | array | `[]` | Sous-éléments rendus dans l'ordre |

#### `frame`
Encadré avec bordure et variants visuels.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `title` | string | `""` | Titre |
| `variant` | `"flat"\|"terminal"\|"bezel"\|"glow"` | `"flat"` | Style visuel |
| `led` | `"red"\|"green"\|"yellow"\|"blue"` | `""` | LED décorative (coin supérieur droit) |
| `padding` | float | `8.0` | Padding interne |
| `children` | array | `[]` | Sous-éléments |

#### `grid`
Grille flexible (row). Les enfants doivent être de type `column`.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `cols` | int | `1` | Nombre total de colonnes |
| `gap` | float | `8.0` | Espacement entre colonnes |
| `children` | array | `[]` | Éléments de type `column` |

Une `column` dans une grid :
| Clé | Type | Défaut | Description |
|---|---|---|---|
| `width` | int | `1` | Largeur relative (sur `cols`) |
| `children` | array | `[]` | Éléments rendus dans cette colonne |

#### `v_stack`
Colonne flexible simple. Aucun paramètre.

#### `h_split`
Deux colonnes côte à côte (50/50). Aucun paramètre.

#### `overlay`
Effet visuel superposé.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `effect` | `"scanlines"` | `""` | Effet |
| `opacity` | float | `0.2` | Opacité |

### 4.2 Texte statique

#### `label`
Texte fixe.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `text` | string | `""` | Texte à afficher |
| `style` | `"body"\|"title"\|"small"\|"green"` | `"body"` | Style (taille/couleur depuis le thème) |
| `font_size` | float | — | Surcharge la taille du style |
| `color` | string (hex) | — | Surcharge la couleur du style |

#### `data_label`
Texte résolu depuis le contexte de données.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `key` | string | `""` | Clé de donnée (ex: `game_over.waves`) |
| `style` | `"body"\|"green"\|"yellow"` | `"body"` | Style |

### 4.3 Texte dynamique

#### `hud_text`
Texte HUD positionné absolument (hors flux).

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `data_key` | string | `""` | Clé de donnée |
| `font_size` | float | — | Taille en pixels |
| `color` | string (hex) | — | Couleur |
| `position` | table | — | `{ top, right, bottom, left }` en pixels |

#### `data_text`
Texte mis à jour par sélection (logs capsule).

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `data_key` | string | `""` | Clé de donnée |

#### `conditional_text`
Affiche un texte si une condition est vérifiée.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `key` | string | `""` | Clé à résoudre |
| `match` | string | `""` | Valeur attendue |
| `text` | string | `""` | Texte affiché si `data.resolve(key) == match` |

#### `badge_list`
Liste de badges (utilisé pour les phases capsule).

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `data_key` | string | `""` | Clé de donnée |

#### `animate`
Animation sur l'élément parent.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `effect` | `"pulse"\|"glitch"` | — | Effet d'animation |
| `duration` | float | `1.0` | Durée en secondes |

### 4.4 Boutons

#### `button`
Bouton cliquable générique.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `text` | string | `"Button"` | Texte du bouton |

#### `active_toggle`
Toggle Actif/Inactif pour panneau de bâtiment. Aucun paramètre.

### 4.5 Barres et progression

#### `progress_bar`
Barre de progression data-driven.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `key` | string | `"0"` | Clé pour la valeur courante |
| `max_key` | string | `"100"` | Clé pour la valeur max |

#### `hp_bar`
Barre de vie (utilise les clés `hp.current`, `hp.max` du contexte).

#### `tier_progress`
Progression de tier (capsule).

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `current_key` | string | — | Clé pour le tier courant |
| `max_key` | string | — | Clé pour le tier max |

#### `recipe_progress`
Barre de progression + timer pour recette en cours. Utilise `recipe.progress` et `recipe.time_sec` du contexte.

### 4.6 Grille d'inventaire

#### `inventory_grid`
Grille de slots d'inventaire. L'entité propriétaire est celle du `UiDataContext`.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `cols` | int | `3` | Nombre de colonnes |
| `rows` | int | `2` | Nombre de lignes |

### 4.7 Artisanat

#### `hand_crafting_list`
Liste des recettes craftable à la main, avec boutons "Craft".
Filtre les recettes dont `craftable_in` contient `"hand"` et débloquées dans l'archive globale.
Aucun paramètre.

#### `hand_crafting_progress`
Affiche la progression de la recette manuelle en cours (texte).
Mis à jour chaque frame par le système `update_hand_crafting_progress`.
Aucun paramètre.

### 4.8 Recettes (panneaux bâtiments)

#### `recipe_name`
Nom de la recette active du bâtiment inspecté. Aucun paramètre.
Utilise `building.name` et `recipe.name` du contexte.

#### `recipe_category`
Liste des recettes d'une catégorie, avec clic pour changer la recette active.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `category` | string | `""` | Catégorie de recettes à afficher (ex: `"smelting"`) |

#### `recipe_progress`
Barre de progression + timer de la recette en cours.
Utilise `recipe.progress` et `recipe.time_sec`.

### 4.9 Misc

#### `spacer`
Espacement vertical.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `height` | float | `8.0` | Hauteur en pixels |

#### `icon`
Icône de ressource.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `icon` | string | — | ID de la ressource |
| `size` | float | `24.0` | Taille en pixels |

#### `wireframe`
Wireframe 2D (schéma capsule).

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `width` | float | — | Largeur |
| `height` | float | — | Hauteur |
| `shapes` | array | `[]` | Formes : `ellipse` (cx,cy,rx,ry,color_key), `rect` (x,y,w,h,color_key), `hline` (y,x1,x2,color_key), `vline` (x,y1,y2,color_key) |

#### `key_value`
Paire clé-valeur.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `key` | string | — | Texte fixe de la clé |
| `value_key` | string | — | Clé de donnée pour la valeur |

#### `key_value_list`
Liste de paires clé-valeur.

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `items` | array | `[]` | Chaque item : `{ key, value_key }` |

#### `alert_header`
Entête d'alerte (capsule).

| Clé | Type | Défaut | Description |
|---|---|---|---|
| `alert` | string | — | Texte d'alerte |
| `subtitle_key` | string | — | Clé de donnée pour le sous-titre |

#### `build_bar`
Conteneur pour la barre de construction. Le contenu (boutons, navigation) est encore géré en Rust. Aucun paramètre.

---

## 5. Clés de données disponibles

Ces clés sont résolues via `UiDataContext` au moment du rendu.

### Panel capsule
| Clé | Valeur |
|---|---|
| `capsule.status_power` | Statut alimentation |
| `capsule.status_cooling` | Statut refroidissement |
| `capsule.status_heart` | Statut cœur biologique |
| `capsule.status_glass` | Statut vitre capsule |
| `capsule.current_tier` | Tier actuel |
| `capsule.total_tiers` | Nombre total de tiers |
| `capsule.phase_list` | Liste des phases débloquées |
| `capsule.logs` | Liste des logs de progression |
| `capsule.log_text` | Texte du log sélectionné |

### Panel game over
| Clé | Valeur |
|---|---|
| `game_over.waves` | Nombre de vagues survécues |

### Panel HUD
| Clé | Valeur |
|---|---|
| `hud.wave_counter` | Vague actuelle |
| `hud.fps` | Images par seconde |

### Panel bâtiment
| Clé | Valeur |
|---|---|
| `building.name` | Nom du bâtiment |
| `building.description` | Description |
| `building.tier` | Tier actuel |
| `building.max_tier` | Tier maximum |
| `building.hp_current` | PV actuels |
| `building.hp_max` | PV max |
| `building.objective` | Objectif actuel |
| `recipe.name` | Nom de la recette active |
| `recipe.progress` | Progression (0.0–1.0) |
| `recipe.time_sec` | Temps total en secondes |
| `hp.current` | PV (générique) |
| `hp.max` | PV max (générique) |

---

## 6. Règles et contraintes

- Les couleurs s'écrivent en hexadécimal avec `#` : `#ffcc44`, `#33ff33`, etc. (6 chiffres obligatoire).
- `font_size` est en pixels.
- Les `position` des `hud_text` acceptent `top`, `right`, `bottom`, `left` en pixels.
- Les `elements` et `children` sont des tableaux TOML évalués séquentiellement.
- Si un type de composant est inconnu, le moteur log un warning et ignore l'élément (pas de crash).
- Les fichiers sont chargés via `ModRegistry::load_data("panel_{name}.toml")` — placez-les dans `data/` de votre mod.
- Le système de thème (`Theme`) fournit les couleurs/base. Les composants peuvent les surcharger via des paramètres TOML.

---

## 7. Fonctionnalités encore en Rust (non personnalisables en TOML)

Ces zones sont encore en code Rust et ne peuvent pas être modifiées via TOML. Elles seront migrées progressivement :

| Fonctionnalité | Fichier(s) Rust |
|---|---|
| Contenu de la barre de construction | `economy/build_bar/` |
| Toasts (notifications) | `core/toast.rs` |
| Tooltips | `core/tooltip.rs` |
| Menu principal | `core/main_menu/` |
| Drag & drop inventaire | `economy/ui.rs` |
| Drag de fenêtre | `economy/window.rs` |
| Menu pause / save / load | `save_load/` |
| Panneau d'artisanat (bouton Craft) | `player/crafting.rs` |

---

## 8. Exemple complet : Panel capsule

```toml
title = "GENESIS ARRAY — STATUS TERMINAL"
width = 960
height = 720

[[sections]]
type = "alert_header"
alert = "SÉQUENCE DE STASE : DÉFAILLANCE CRITIQUE"
subtitle_key = "objective.current"

[[sections]]
type = "grid"
cols = 12
gap = 8

[[sections.children]]
type = "column"
width = 8
children = [
    { type = "frame", title = "CAPSULE SCHEMATIC", variant = "terminal", led = "red", children = [
        { type = "overlay", effect = "scanlines", opacity = 0.2 },
        { type = "wireframe", width = 600, height = 280, shapes = [
            { type = "ellipse", cx = 300, cy = 140, rx = 130, ry = 80, color_key = "#33ff33" },
            { type = "rect", x = 220, y = 100, w = 160, h = 80, color_key = "#33ff33" },
            { type = "hline", y = 230, x1 = 80, x2 = 520, color_key = "#ffaa00" },
        ]},
        { type = "animate", effect = "pulse", duration = 2.0 },
    ]},
    { type = "frame", title = "SYSTÈMES", children = [
        { type = "key_value_list", items = [
            { key = "⚠ ALIMENTATION", value_key = "capsule.status_power" },
            { key = "⚠ REFROIDISSEMENT", value_key = "capsule.status_cooling" },
            { key = "⚠ CŒUR BIOLOGIQUE", value_key = "capsule.status_heart" },
        ]},
    ]},
]

[[sections.children]]
type = "column"
width = 4
children = [
    { type = "frame", title = "PROGRESSION", children = [
        { type = "tier_progress", current_key = "capsule.current_tier", max_key = "capsule.total_tiers" },
        { type = "badge_list", data_key = "capsule.phase_list" },
    ]},
]
```

---

## 9. Checklist de déploiement

Avant de livrer votre mod, vérifiez :

- [ ] `panel_hud.toml` présent — sinon pas de HUD en jeu
- [ ] `panel_game_over.toml` présent — sinon écran noir au Game Over
- [ ] `panel_inventory.toml` présent — sinon la touche I ne fait rien
- [ ] `panel_hand_crafting.toml` présent — sinon la touche C ne fait rien
- [ ] `panel_build_bar.toml` présent — sinon pas de barre de construction
- [ ] `panel_capsule.toml` présent — sinon pas de panneau capsule
- [ ] Chaque fichier `panel_{building_id}.toml` correspond à un ID de bâtiment valide dans `buildings.toml`
- [ ] Les clés de données utilisées existent dans le contexte (vérifiez §5)
- [ ] Toutes les couleurs sont en hex 6 chiffres avec `#`
- [ ] `cargo test` passe après vos changements

---

*Document généré depuis le code source — se référer aux fichiers `src/ui/components/*.rs` pour les détails d'implémentation.*
