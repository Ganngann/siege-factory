# 24 — Minimap : bordure data-driven

## Demande

Ajouter une bordure autour de la minimap, configurable depuis `panel_hud.toml`.

Motif : la minimap est une `Camera2d` avec `order: 1` qui écrase le rendu UI dans sa zone de viewport. Impossible de faire une bordure propre côté TOML seul — besoin d'un nouveau composant Rust.

## TOML attendu (déjà dans `panel_hud.toml`)

```toml
[[sections]]
type = "minimap"
size = 200
margin = 10
border = { width = 2, color = "#33ff33" }
```

- `size` : taille en pixels (carré)
- `margin` : marge depuis le bord de l'écran (bas-droite)
- `border.width` : épaisseur en pixels
- `border.color` : couleur hex (`#RRGGBB`)

## Nouveau composant Rust

### `src/ui/components/minimap.rs`

Implémenter `UiComponent` pour `id() = "minimap"`.

**`render()` doit :**
1. Lire `size`, `margin`, `border.width`, `border.color` du `config` TOML
2. Spawner la `Camera2d` + `MinimapCamera` (actuellement dans `setup_minimap` dans `rendering/minimap.rs`)
3. Spawner la bordure VISIBLE par-dessus la minimap

### Contrainte de rendu

La minimap caméra (order 1) écrase le rendu UI dans sa zone. Donc une bordure en UI Node sera invisible.

**Solutions côté rendu (au choix du dev) :**

1. **RenderLayers** : 
   - Main camera → `RenderLayers::layer(0)`
   - Minimap camera → `RenderLayers::from_layers(&[0, 1])`
   - Bordure en 4 sprites (rectangles fins) enfants de la minimap camera → `RenderLayers::layer(1)`
   - La bordure suit la caméra automatiquement (children en local space)

2. **Caméra overlay (order 2)** :
   - 3e caméra avec même viewport, `ClearColorConfig::None`, order 2
   - La bordure est un enfant UI ou sprite de cette caméra
   - `update_minimap` synchronise la position du viewport des 2 caméras

3. **Autre** au choix du dev — l'important c'est que la bordure soit visible.

### `MinimapBorderConfig` component

Stocke `size` et `margin` pour que `update_minimap` puisse les lire au lieu des constantes 200/10.

```rust
#[derive(Component)]
pub struct MinimapBorderConfig {
    pub size: u32,
    pub margin: u32,
}
```

Sera attaché à l'entité caméra minimap par le composant TOML, et lu par `update_minimap`.

## Modifications fichier par fichier

### Créer

| Fichier | Contenu |
|---------|---------|
| `src/ui/components/minimap.rs` | `MinimapComponent` (UiComponent), `MinimapBorderConfig` |

### Modifier

| Fichier | Changement |
|---------|-----------|
| `src/ui/components/mod.rs` | Ajouter `pub mod minimap;` |
| `src/ui/mod.rs` | Importer + enregistrer `MinimapComponent` |
| `src/rendering/minimap.rs` | Supprimer `setup_minimap`. `update_minimap` lit `MinimapBorderConfig` au lieu des constantes 200/10. |
| `src/rendering/mod.rs` | Supprimer `minimap::setup_minimap` de `OnEnter(Playing)` |
| `mods/genesis_protocol/data/panel_hud.toml` | DÉJÀ FAIT — section `minimap` présente |

### Aucun changement

- `panel_capsule.toml` — pas touché
- `src/save_load/load.rs` — pas touché (la main camera garde ses réglages par défaut)
