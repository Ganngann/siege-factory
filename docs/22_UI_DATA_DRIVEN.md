# 22 — Panneau capsule et composants visuels (data-driven)

Guide d'utilisation de chaque fonctionnalité.

---

## 1. Nouveaux composants TOML

### `alert_header`

Bannière d'alerte en haut du panneau.

```toml
{ type = "alert_header", title = "GENESIS ARRAY", alert = "ALERTE CRITIQUE", subtitle_key = "objective.current" }
```

| Paramètre | Valeurs | Description |
|-----------|---------|-------------|
| `title` | string | Titre principal |
| `alert` | string | Texte dans la bannière rouge (optionnel) |
| `subtitle_key` | string | Clé de donnée pour le sous-titre |

---

### `frame`

Conteneur avec bordure, fond, titre optionnel et LED.

```toml
{ type = "frame", title = "TITRE", variant = "terminal", led = "red", padding = 8, children = [...] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `title` | string | `""` | Titre affiché en haut |
| `variant` | `flat`, `terminal`, `bezel`, `glow` | `flat` | Style visuel |
| `led` | `red`, `green`, `yellow`, `blue`, `""` | `""` | LED colorée en haut à droite |
| `padding` | float | `8` | Padding intérieur |
| `children` | array | — | Composants enfants |

Les variantes changent les couleurs de fond et bordure :
- `terminal` → fond bleu foncé, bordure verte
- `bezel` → fond gris foncé, bordure bleu-gris
- `glow` → fond bleu nuit, bordure bleu clair
- `flat` → fond sombre, bordure grise

---

### `overlay`

Calque semi-transparent (scanlines ou vignette).

```toml
{ type = "overlay", effect = "scanlines", opacity = 0.2 }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `effect` | `scanlines`, `vignette`, `none` | `none` | Effet visuel |
| `opacity` | float (0-1) | `0.25` | Opacité |

Placement absolu sur le parent (parfait pour superposer à un `frame`).

---

### `animate`

Animation cyclique sur les enfants (pulse, blink, flicker).

```toml
{ type = "animate", effect = "pulse", duration = 2.0, children = [...] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `effect` | `pulse`, `blink`, `flicker`, `none` | `none` | Type d'animation |
| `duration` | float (secondes) | `1.0` | Durée d'un cycle |
| `children` | array | — | Composants à animer |

Animations :
- `pulse` → fondu sinusoïdal (alpha 0.65 → 1.0)
- `blink` → clignotement binaire (50% on/off)
- `flicker` → scintillement aléatoire (type néon défaillant)

---

### `wireframe`

Dessin de formes primitives (lignes, rectangles, ellipses).

```toml
{ type = "wireframe", width = 400, height = 200, shapes = [
    { type = "ellipse", cx = 200, cy = 100, rx = 80, ry = 50, color_key = "#33ff33" },
    { type = "rect", x = 160, y = 80, w = 80, h = 40, color_key = "#33ff33" },
    { type = "hline", y = 150, x1 = 120, x2 = 280, color_key = "#ffaa00" },
    { type = "vline", x = 120, y1 = 60, y2 = 150, color_key = "#ffaa00" },
] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `width` | float | `400` | Largeur du conteneur |
| `height` | float | `200` | Hauteur du conteneur |
| `shapes` | array | — | Liste de formes |

**Formes disponibles :**

#### ellipse
```toml
{ type = "ellipse", cx = 200, cy = 100, rx = 80, ry = 50, color_key = "#33ff33" }
```
- `cx`, `cy` : centre
- `rx`, `ry` : rayons horizontal et vertical
- rendu : bordure de 1px

#### rect
```toml
{ type = "rect", x = 160, y = 80, w = 80, h = 40, color_key = "#33ff33" }
```
- `x`, `y` : position haut-gauche
- `w`, `h` : dimensions
- rendu : plein

#### hline / vline
```toml
{ type = "hline", y = 150, x1 = 120, x2 = 280, color_key = "#ffaa00" }
{ type = "vline", x = 120, y1 = 60, y2 = 150, color_key = "#ffaa00" }
```
- `hline` : `y` (hauteur), `x1`, `x2` (début/fin horizontale)
- `vline` : `x` (colonne), `y1`, `y2` (début/fin verticale)

Le `color_key` est mis à jour dynamiquement par `update_capsule_wireframe_system` selon le tier actuel de la capsule (couleur du statut `power`).

---

### `grid`

Grille multi-colonnes avec colonnes de largeur variable.

```toml
{ type = "grid", cols = 12, gap = 8, children = [
    { type = "column", width = 8, children = [ ... ] },
    { type = "column", width = 4, children = [ ... ] },
] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `cols` | integer | `1` | Nombre total de colonnes (pour le calcul des %) |
| `gap` | float | `8` | Espacement entre colonnes |
| `children` | array | — | Liste d'éléments de type `column` |

Chaque enfant doit être `{ type = "column", width = N, children = [...] }` où `width` est le nombre de colonnes que cette colonne occupe. Le pourcentage est calculé automatiquement.

---

### `key_value_list`

Liste de paires clé/valeur liées aux statuts capsule.

```toml
{ type = "key_value_list", items = [
    { key = "ALIMENTATION", value_key = "capsule.status_power" },
    { key = "REFROIDISSEMENT", value_key = "capsule.status_cooling" },
] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `items` | array | — | Liste d'entrées |

Chaque item :
- `key` : texte fixe affiché à gauche
- `value_key` : clé préfixée par `capsule.status_` suivie du nom du système (ex: `power`, `cooling`). Le système `update_capsule_statuses_system` résout le texte et la couleur depuis `CapsuleStatusRegistry`.

Le composant `CapsuleStatusRow` stocke le `system_id` extrait du `value_key`, et `update_capsule_statuses_system` met à jour le `Text` et `TextColor` en fonction du tier actuel de la capsule.

---

### `key_value` (unitaire)

Paire clé/valeur simple.

```toml
{ type = "key_value", key = "ALIMENTATION", value_key = "capsule.status_power" }
```

Mêmes paramètres que ci-dessus, mais sans le conteneur `CapsuleStatusList`. La valeur est résolue via `UiDataContext.resolve()`.

---

### `badge_list`

Liste de badges (phases/progression) avec état.

```toml
{ type = "badge_list", data_key = "capsule.phase_list" }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `data_key` | string | `""` | Clé de donnée contenant un TOML `items = [...]` |

Le contenu de la clé doit être un TOML avec ce format :
```toml
items = [
    { id = "0", title = "Phase 0", state = "done" },
    { id = "1", title = "Phase 1", state = "current", separator = true },
    { id = "2", title = "Phase 2", state = "locked" },
]
```

États :
- `done` → fond vert, `[✓]`
- `current` → fond rouge, `[N]`
- `locked` → fond gris foncé, `[N]`
- `separator = true` → affiche `[!]` (current) ou `[ ]` (locked)

Les données sont pré-résolues dans `building_inspect_click` sous la clé `capsule.phase_list`.

---

### `tier_progress`

Barre de progression avec indicateurs visuels de tier.

```toml
{ type = "tier_progress", current_key = "capsule.current_tier", max_key = "capsule.total_tiers" }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `current_key` | string | `"0"` | Clé pour le tier actuel |
| `max_key` | string | `"1"` | Clé pour le nombre total de tiers |

Rend : `███████░░ 5/8`

---

### `icon`

Icône unicode.

```toml
{ type = "icon", name = "warning", color = "red", size = 16 }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `name` | `warning`, `alert`, `check`, `cross`, `info`, `heart`, `power` | `?` | Nom de l'icône |
| `color` | `red`, `green`, `yellow`, `currentColor`, ou hex `#rrggbb` | `currentColor` | Couleur |
| `size` | float | `16` | Taille en pixels |

---

### `badge_list` → `data_list` / `data_text`

Ensemble liste cliquable + texte détaillé pour les logs de progression.

```toml
{ type = "data_list", data_key = "capsule.logs" }
{ type = "data_text", data_key = "capsule.log_text" }
```

- `data_list` : liste cliquable d'éléments (chaque ligne est un `Button` avec `DataListItem`). Le clic met à jour `DataListSelected`.
- `data_text` : texte détaillé mis à jour par `update_data_text_system` quand `DataListSelected` change.
- Les données proviennent de `ProgressionLogRegistry`.

---

### `conditional_text`

Affiche un texte conditionnel basé sur la valeur d'une clé.

```toml
{ type = "conditional_text", source_key = "building.kind", values = [
    { when = "assembler", text = "Assemble des composants" },
    { when = "furnace", text = "Fond des minerais" },
] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `source_key` | string | `""` | Clé de donnée source |
| `values` | array | — | Liste `{ when, text }` |

Si `when` correspond à la valeur résolue de `source_key`, le texte associé est affiché. Sinon, chaîne vide.

---

## 2. Nouvelles clés de données disponibles

Clés ajoutées dans `UiDataContext` pour le panneau capsule :

| Clé | Source | Type |
|-----|--------|------|
| `capsule.current_tier` | `CurrentTier` du `Capsule` | `"5"` |
| `capsule.total_tiers` | `BuildingDef.tiers.len()` | `"8"` |
| `capsule.phase_list` | Pre-résolu en TOML (liste de phases) | TOML string |
| `capsule.status_power` | Résolu par `update_capsule_statuses_system` | texte/couleur |
| `capsule.status_cooling` | Idem | texte/couleur |
| `capsule.status_heart` | Idem | texte/couleur |
| `capsule.status_glass` | Idem | texte/couleur |
| `capsule.logs` | `ProgressionLogRegistry.unlocked` | Liste cliquable |
| `capsule.log_text` | Texte du log sélectionné | Texte |
| `objective.current` | `ObjectiveState.active_text` (caché dans `BuildingPanel.cached_objective`) | Texte |

---

## 3. CapsuleStatusRegistry (`capsule_status.toml`)

Fichier data-driven définissant les textes et couleurs par système/tier.

```toml
[[statuses]]
id = "power"
label = "ALIMENTATION"

[[statuses.tiers]]
tier = 0
text = "DÉFAILLANTE"
color = "#ff3333"

[[statuses.tiers]]
tier = 1
text = "ALIMENTATION PRIMAIRE"
color = "#ffaa00"
# ... plus de tiers
```

- Chaque `[[statuses]]` a un `id` (référencé par `value_key = "capsule.status_{id}"`)
- Chaque `[[statuses.tiers]]` a `tier` (index 0-based), `text`, `color` (hex `#rrggbb`)
- Le système `update_capsule_statuses_system` cherche le tier le plus élevé ≤ `CurrentTier` et met à jour le texte/couleur

**Pour ajouter un nouveau système :**
1. Ajouter `[[statuses]]` dans `capsule_status.toml` avec ses tiers
2. Ajouter `{ key = "NOM", value_key = "capsule.status_{id}" }` dans `panel_capsule.toml`

---

## 4. Systèmes de mise à jour temps réel

### `update_capsule_statuses_system`

- S'exécute chaque frame
- Lit `CapsuleStatusRegistry` + `CurrentTier` de l'entité inspectée
- Met à jour le `Text` et `TextColor` de chaque `CapsuleStatusRow`
- Ne nécessite pas de re-rendu du panneau

### `update_capsule_wireframe_system`

- Met à jour la couleur des formes `wireframe` selon le tier
- Utilise la couleur du statut `power` depuis `CapsuleStatusRegistry`
- Cible `WireframeTierTracker` sur le conteneur wireframe

### `animation_tick_system`

- Anime les composants `AnimationState` (pulse/blink/flicker)
- Modifie l'alpha du `BackgroundColor` en fonction du temps
- Cycle continu (timer remis à zéro à `duration`)

---

## 5. `build.rs` — Propriétés data-driven

Les composants sont insérés par propriétés `BuildingDef`, pas par ID.

| Propriété `BuildingDef` | Composant inséré |
|-------------------------|------------------|
| `has_recipes` / `default_recipe` / `production_interval` | `Assembler` ou `RecipeGenerator` + `ProductionCounter` + `DiscoveredRecipes` |
| `combat` (Some) | `TurretCombat` |
| `crop_types` (non vide) | `Farm` |
| `is_archive` (true) | `Archive` |
| `compactor_ratio` (> 0) | `Compactor` |
| `fluid_tank_capacity` (> 0) | `FluidTank` (et `Pump` si `id == "water_pump"`) |
| `pipe_transfer_rate` (> 0) | `FluidPipe` |
| `power_consumption` (> 0) | `PowerConsumer` |
| `power_generation` (> 0) | `PowerProducer` |
| `fuel_burn_interval` (> 0) | `BurnerGenerator` |
| `power_pole_range` (> 0) | `PowerPole` |

**Avantage :** un mod peut définir n'importe quelle combinaison de ces propriétés dans `buildings.toml` sans toucher au code Rust. Exemple : un bâtiment avec recettes ET combat ET stockage fonctionne immédiatement.

---

## 6. Architecture du panneau capsule (exemple complet)

Fichier : `mods/genesis_protocol/data/panel_capsule.toml`

```toml
title = "GENESIS ARRAY — STATUS TERMINAL"
width = 960
height = 720

[[sections]]
type = "alert_header"
title = "GENESIS ARRAY — STATUS TERMINAL"
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
        { type = "wireframe", shapes = [ ... ] },
        { type = "animate", effect = "pulse", duration = 2.0 },
    ] },
    { type = "frame", title = "SYSTÈMES", children = [
        { type = "key_value_list", items = [ ... ] },
        { type = "icon", name = "warning", color = "red" },
    ] },
]

[[sections.children]]
type = "column"
width = 4
children = [
    { type = "frame", title = "PROGRESSION", children = [
        { type = "tier_progress", current_key = "capsule.current_tier", max_key = "capsule.total_tiers" },
        { type = "badge_list", data_key = "capsule.phase_list" },
    ] },
]
```

**Structure hiérarchique :**
- `grid` (ligne) → `column` (colonnes redimensionnables)
- `frame` (bordures + LED) → enfants overlay + wireframe + animate
- `key_value_list` (statuts dynamiques mis à jour chaque frame)
- `tier_progress` + `badge_list` (progression statique lue du cache)

---

## 7. Ajouter un panneau TOML pour un nouveau bâtiment

1. Créer `mods/genesis_protocol/data/panel_maison.toml` :

```toml
title = "Ma Maison"
width = 400
height = 300

[[sections]]
type = "section"
title = "INFOS"
elements = [
    { type = "data_label", key = "building.name", style = "title" },
    { type = "data_label", key = "hp.current", style = "body" },
]

[[sections]]
type = "hp_bar"
```

2. Ajouter dans `buildings.toml` :

```toml
[[buildings]]
id = "ma_maison"
name = "Ma Maison"
panel = "maison"
```

Le `LayoutEngine` charge automatiquement `panel_maison.toml`.

---

## 8. Anciens fichiers supprimés

Suite à la migration data-driven, ces fichiers ont été supprimés :

| Fichier | Remplacé par |
|---------|-------------|
| `recipe_selector.rs` | `recipe_category.rs` + `recipe_row.rs` |
| `crafting.rs` | `recipe_progress.rs` + `recipe_name.rs` |
| `building_panel_ui.rs` | Panneaux TOML + `building.rs` (ui/panels) |
| `deposit.rs` | `panel_compactor.toml` |
| `RecipeChangeButton` | `RecipeRow` (clic) |
| `RecipeSelectorItem / RecipeSelectorRoot / RecipeSelectorRoot` | Supprimés |
| `DataPadEntry / DataPadFullText / DataPadSelected` | `data_list` + `data_text` |
| `PanelModal` | Supprimé (overlay click simplifié) |
| `Storage` component | Remplacé par `Inventory::with_capacity` |
| `BUILDING_*` constantes | `BuildingDef` properties |
