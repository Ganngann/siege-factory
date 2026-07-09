# Composants UI disponibles

Tous les composants sont enregistrés dans `ComponentRegistry` au démarrage du jeu.
Utilisables depuis n'importe quel fichier TOML de panneau via `{ type = "..." }`.

---

## label

Texte stylisé.

```toml
{ type = "label", text = "Hello", style = "body" }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `text` | string | `""` | Texte à afficher |
| `style` | `title`, `body`, `small`, `accent`, `green` | `body` | Style de texte |

**Styles** :
- `title` → police 16px, couleur primaire
- `body` → police 12px, couleur primaire
- `small` → police 10px, couleur secondaire
- `accent` → police 12px, couleur accent (bleu)
- `green` → police 12px, couleur verte

---

## data_label

Texte lié à une valeur du jeu. Résout automatiquement la donnée via `UiDataContext`.

```toml
{ type = "data_label", key = "building.name", style = "body" }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `key` | string | `""` | Clé de donnée à résoudre |
| `style` | `body`, `green`, `yellow` | `body` | Style de couleur |

**Clés disponibles** :

| Clé | Résultat | Source |
|-----|----------|--------|
| `entity.id` | ID numérique de l'entité | `Entity::to_bits()` |
| `building.name` | Nom du bâtiment | `Building.name` |
| `building.kind` | Type du bâtiment | `Building.kind` |
| `active` | `"ON"` ou `"OFF"` | `Active` component |
| `inventory.total` | Total items dans l'inventaire | `Inventory.total()` |
| `inventory.capacity` | Capacité inventaire | `Inventory.capacity` |
| `hp.current` | PV actuels | `Health.current` |
| `hp.max` | PV maximum | `Health.max` |

---

## section

Section encadrée avec titre optionnel + éléments enfants.

```toml
{ type = "section", title = "STATS", elements = [
    { type = "label", text = "Production: 10/min" },
    { type = "progress_bar", key = "fuel", max_key = "100" }
] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `title` | string | `""` | Titre de la section |
| `elements` | array | `[]` | Composants enfants à rendre dans la section |

---

## h_split

Split horizontal : deux colonnes côte à côte.

```toml
{ type = "h_split", left_width = 58, right_width = 38,
  left = [ { type = "inventory_grid", cols = 3, rows = 2 } ],
  right = [ { type = "active_toggle" } ]
}
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `left_width` | float | `58` | Largeur colonne gauche (%) |
| `right_width` | float | `38` | Largeur colonne droite (%) |
| `left` | array | — | Composants colonne gauche |
| `right` | array | — | Composants colonne droite |

---

## v_stack

Pile verticale d'éléments.

```toml
{ type = "v_stack", children = [
    { type = "label", text = "Ligne 1" },
    { type = "label", text = "Ligne 2" }
] }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `children` | array | `[]` | Composants à empiler |

---

## spacer

Espacement vertical.

```toml
{ type = "spacer", height = 16 }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `height` | float | `8` | Hauteur en pixels |

---

## progress_bar

Barre de progression avec valeur texte.

```toml
{ type = "progress_bar", key = "production.current", max_key = "production.max" }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `key` | string | `"0"` | Clé pour la valeur courante |
| `max_key` | string | `"100"` | Clé pour la valeur maximale |

Rend : `[████░░░░░░] 50/100`

---

## hp_bar

Barre de vie.

```toml
{ type = "hp_bar" }
```

Utilise les clés `hp.current` et `hp.max` du `UiDataContext`.

Rend : `HP: 75/100`

---

## inventory_grid

Grille de slots d'inventaire.

```toml
{ type = "inventory_grid", cols = 3, rows = 2 }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `cols` | integer | `3` | Nombre de colonnes |
| `rows` | integer | `2` | Nombre de lignes |

Le nombre total de slots = `cols × rows`. Chaque slot est un `Button` cliquable avec le composant `InventorySlot`.
L'owner de la grille est l'entité inspectée (`UiDataContext.entity`).

---

## active_toggle

Bouton ON/OFF pour activer/désactiver un bâtiment.

```toml
{ type = "active_toggle" }
```

- Si `active = "ON"` (via `UiDataContext`) → bouton "⏸ Pause" (vert)
- Si `active = "OFF"` → bouton "▶ Activer" (rouge)
- Le clic est géré par le système existant `active_toggle_system`

---

## button

Bouton cliquable générique.

```toml
{ type = "button", text = "Cliquez-moi" }
```

| Paramètre | Valeurs | Défaut | Description |
|-----------|---------|--------|-------------|
| `text` | string | `"Button"` | Texte du bouton |

Note : les actions du bouton doivent être gérées par un système Rust séparé écoutant l'interaction.
