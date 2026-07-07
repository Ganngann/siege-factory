# Map Generation — Siege Factory

## Architecture

La carte est générée à partir d'une configuration data-driven (position des dépôts, dimensions, seed, tailes).

- Grille tuilée configurable
- Gisements placés selon la config
- Génération procédurale avec seed déterministe (destination)
- Découpage en chunks pour carte infinie (destination)

## Tests

- Seed fixe → carte identique
- Pas de gisement sur le point de départ
- Tous les gisements sont dans la carte
