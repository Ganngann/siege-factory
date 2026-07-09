# Exemples de panneaux TOML

## Générique — Tout bâtiment

```toml
title = "{building.name}"
width = 800
height = 560

[[sections]]
type = "h_split"
left_width = 58
right_width = 38

[[sections.left]]
type = "inventory_grid"
cols = 3
rows = 2

[[sections.right]]
type = "section"
title = "STATS"
elements = [
    { type = "data_label", key = "building.name" },
    { type = "data_label", key = "active", style = "green" }
]

[[sections.right]]
type = "active_toggle"
```

## Capsule Genesis

```toml
title = "Capsule Genesis"
width = 400
height = 320

[[sections]]
type = "section"
title = "PROGRESSION"
elements = [
    { type = "label", text = "Tier 0 : Déblayage", style = "accent" },
    { type = "label", text = "Tier 1 : Réveil", style = "body" }
]

[[sections]]
type = "section"
title = "ITEMS REQUIS"
elements = [
    { type = "data_label", key = "inventory.total" }
]
```

## Dépôt de ressources

```toml
title = "Dépôt de Ressources"
width = 400
height = 200

[[sections]]
type = "section"
elements = [
    { type = "data_label", key = "building.name", style = "title" },
    { type = "progress_bar", key = "inventory.total", max_key = "inventory.capacity" }
]
```

## Four

```toml
title = "Four"
width = 800
height = 560

[[sections]]
type = "h_split"
left_width = 58
right_width = 38

[[sections.left]]
type = "inventory_grid"
cols = 3
rows = 2

[[sections.left]]
type = "section"
title = "PRODUCTION"
elements = [
    { type = "progress_bar", key = "production.progress", max_key = "100" }
]

[[sections.right]]
type = "active_toggle"

[[sections.right]]
type = "section"
title = "CARBURANT"
elements = [
    { type = "progress_bar", key = "fuel.level", max_key = "100" }
]

[[sections.right]]
type = "hp_bar"
```

## Générateur Électrique (exemple mod)

```toml
title = "Générateur Électrique"
width = 800
height = 560

[[sections]]
type = "h_split"
left_width = 58
right_width = 38

[[sections.left]]
type = "section"
title = "CARBURANT"
elements = [
    { type = "inventory_grid", cols = 2, rows = 2 },
    { type = "progress_bar", key = "fuel.level", max_key = "100" }
]

[[sections.right]]
type = "active_toggle"

[[sections.right]]
type = "section"
title = "PUISSANCE"
elements = [
    { type = "data_label", key = "power.output", style = "green" },
    { type = "data_label", key = "power.max", style = "body" }
]

[[sections.right]]
type = "hp_bar"
```
