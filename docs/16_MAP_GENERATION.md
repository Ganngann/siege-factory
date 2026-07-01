# Map Generation — Siege Factory

## Principe

Génération procédurale de la carte au début de chaque partie. Seed déterministe : même seed = même carte.

## Éléments générés

| Élément | Distribution | Quantité |
|---|---|---|
| Terrain | Damier / bruit simple | Grille complète |
| Gisements Ore | Clusters aléatoires | 5-15 par carte |
| Spawn ennemis | Bords de carte | 1-4 zones |

## Algorithme (MVP)

1. Grille vide de taille `W × H`
2. Placement du HQ au centre
3. Génération de X gisements d'Ore placés aléatoirement (éviter le HQ)
4. Placement des zones de spawn ennemi sur les bords (N, S, E, O)
5. Le reste est du terrain constructible

```rust
fn generate_map(seed: u64, grid: &mut TileGrid) -> MapData {
    let mut rng = StdRng::seed_from_u64(seed);

    // Placer HQ
    let hq_pos = TilePosition { x: grid.width() / 2, y: grid.height() / 2 };
    grid.set_tile(hq_pos.x, hq_pos.y, TileType::HQ);

    // Placer gisements
    for _ in 0..rng.gen_range(5..15) {
        let pos = random_valid_pos(&mut rng, grid, &[TileType::Ground]);
        grid.set_tile(pos.x, pos.y, TileType::Resource);
    }

    // Placer spawners
    for _ in 0..rng.gen_range(1..4) {
        let pos = random_edge_pos(&mut rng, grid);
        grid.set_tile(pos.x, pos.y, TileType::Spawner);
    }

    MapData { hq_pos, spawners, deposits }
}
```

## Évolution

- **M2** : placement manuel (pas de génération, carte fixe)
- **M4** : génération basique (seed, gisements aléatoires)
- **M6+** : bruit, biomes, obstacles naturels, water

## Paramètres ajustables

Définis dans `data/map_config.toml` (ou settings) :

```toml
[map]
width = 20
height = 15
deposit_min = 5
deposit_max = 15
spawner_count = [1, 4]
hq_clearance = 3    # tuiles libres autour du HQ
```

## Tests

- Seed fixe → carte identique
- Pas de gisement sur le HQ
- Les gisements sont tous dans la carte
- Au moins un spawner
