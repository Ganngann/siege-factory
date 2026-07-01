# Asset Pipeline — Siege Factory

## Structure

```
assets/
├── textures/
│   ├── tiles/
│   │   ├── ground_light.png
│   │   ├── ground_dark.png
│   │   ├── ore_deposit.png
│   │   └── spawner.png
│   ├── buildings/
│   │   ├── hq.png
│   │   ├── miner.png
│   │   ├── assembler.png
│   │   ├── belt.png
│   │   ├── turret.png
│   │   └── wall.png
│   ├── units/
│   │   ├── runner.png
│   │   ├── tank.png
│   │   └── boss.png
│   ├── items/
│   │   ├── ore.png
│   │   └── ammo.png
│   └── ui/
│       ├── button_build.png
│       ├── panel_bg.png
│       └── icons/
│           ├── icon_ore.png
│           └── icon_ammo.png
├── tilesets/
│   └── terrain.ron          # Définition tileset (atlas)
└── fonts/
    └── monogram.ttf         # Police pixel-art
```

## Conventions

- Tous les sprites en PNG. Format recommandé : 16×16 ou 32×32 pixels.
- Les tilesets utilisent le format `.ron` (Rusty Object Notation) pour définir les régions de l'atlas.
- Les placeholders sont des carrés colorés générés en code (aucun asset requis pour le développement).
- Les assets sont chargés via `AssetServer` de Bevy.

## Passage des placeholders aux vrais assets

1. Pendant le prototypage : sprites générés en code (`Color::srgb(...)`).
2. Quand le gameplay est stable : remplacer par des sprites PNG.
3. Le code ne change pas : la logique de rendu utilise des `Handle<Image>` quel que soit le format.

## Gestion des assets en dev

- Ajouter `assets/` au `.gitignore` ? Non. Les assets font partie du projet.
- Les placeholders en code sont la valeur par défaut. Si un asset n'est pas trouvé, le placeholder s'affiche.
- Format `.png` uniquement. Pas de formats compressés (WebP, etc.) pour éviter des dépendances inutiles.

## Tileset définit ion (futur)

```ron
TerrainTileset(
    tile_size: 32,
    columns: 8,
    rows: 8,
    tiles: {
        "ground_light": (0, 0),
        "ground_dark": (1, 0),
        "ore_deposit": (2, 0),
        "spawner": (3, 0),
    }
)
```

Utilisation d'un atlas réduit le nombre de draw calls (recommandé pour 2D avec Bevy).
