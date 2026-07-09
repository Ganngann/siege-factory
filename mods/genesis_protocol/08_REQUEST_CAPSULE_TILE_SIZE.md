# Request 08 — Capsule spawn taille réelle (4×4)

## Contexte

La capsule genesis_ark est créée par `src/economy/capsule.rs` — fonction `spawn_capsule()`. Actuellement, elle est créée en taille fixe 1×1, ignorant le `tile_size` défini dans les TOML.

## Problème

```rust
// capsule.rs lignes 57-74 — spawn en dur à 1×1
custom_size: Some(Vec2::new(tile_size, tile_size)),  // 32×32 fixe
OccupiedTiles(vec![(sx, sy)]),                        // 1 seule tuile
```

Le mod définit `tile_size = { w = 4, h = 4 }` dans `buildings.toml`, mais cette valeur n'est pas lue par `spawn_capsule()`.

## Fix demandé

```rust
// capsule.rs — lire la taille depuis le BuildingRegistry
use crate::economy::building::BuildingRegistry;

let Some(def) = building_registry.get(&capsule_cfg.building_kind) else { return; };
let tw = def.tile_size.0 as f32;
let th = def.tile_size.1 as f32;

// Générer les tuiles occupées
let mut tiles = Vec::with_capacity((tw * th) as usize);
for dx in 0..tw as i32 {
    for dy in 0..th as i32 {
        tiles.push((sx + dx, sy + dy));
    }
}

commands.spawn((
    Capsule,
    CurrentTier(0),
    Building {
        kind: capsule_cfg.building_kind.clone(),
        name: capsule_cfg.building_kind.clone(),
    },
    OccupiedTiles(tiles),
    Inventory::new(),
    TilePosition { x: sx, y: sy },
    Transform::from_xyz(
        pos.x + (tw - 1.0) * cfg.tile_size * 0.5,
        pos.y + (th - 1.0) * cfg.tile_size * 0.5,
        5.0,
    ),
    Visibility::default(),
    Sprite {
        image: tex,
        custom_size: Some(Vec2::new(tw * tile_size, th * tile_size)),
        ..default()
    },
));
```

## Résultat attendu

- La capsule s'affiche en 4×4 tuiles (128×128 pixels jeu) au lieu de 1×1
- Les textures capsules (256×256) sont correctement dimensionnées
- Les autres bâtiments ne peuvent pas être placés dans la zone occupée par la capsule
