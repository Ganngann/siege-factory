# Map Generation — Siege Factory

## Phase actuelle (scaffold TD)

Carte fixe 20×15 définie dans `data/map_config.toml` :

- Tailes : 32.0 px
- Dimensions : 20 × 15
- **6 gisements fixes** (positions définies manuellement)
- HQ position fixe (9, 7)
- 200 Ore de départ
- Zones de spawn ennemi sur les bords

### Gisements

```
positions = [
  { x = 3, y = 3 },  { x = 3, y = 11 },
  { x = 8, y = 6 },  { x = 12, y = 10 },
  { x = 16, y = 4 }, { x = 16, y = 11 },
]
```

Chaque gisement a 50-150 Ore minable.

## Destination (Factorio-like)

- **Génération procédurale** avec seed déterministe
- **Chunks** 32×32, loading/unloading à la volée
- **Biomes** : plaine, forêt, désert, eau, montagnes
- **Ressources réparties** : clusters, veines, poches
- **Carte infinie** : extension dynamique quand le joueur explore

## Paramètres (évolutifs)

Définis dans `data/map_config.toml` :

```toml
[map]
tile_size = 32.0
width = 20          # Sera remplacé par des chunks plus tard
height = 15

[deposits]
min_amount = 50
max_amount = 150
# positions fixes → deviendra procédural

[hq]
start_ore = 200
hp = 100
# position fixe → deviendra placement libre
```

## Tests

- Seed fixe → carte identique (quand la génération sera implémentée)
- Pas de gisement sur le HQ
- Tous les gisements sont dans la carte
- Au moins un spawner
