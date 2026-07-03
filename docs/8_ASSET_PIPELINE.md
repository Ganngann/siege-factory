# Asset Pipeline — Siege Factory

## État actuel

Placeholders générés en code (Mesh2d formes géométriques). Aucun fichier PNG/SVG dans `assets/`.

## Passage aux vrais assets (non décidé)

Le format n'est pas encore choisi (PNG, SVG, ou autre). Deux options :

1. **PNG** : sprites 16×16 ou 32×32, chargés via `AssetServer`
2. **SVG** : rendu vectoriel, plus flexible mais dépendance externe

## Conventions (quand les assets arriveront)

- Tous les sprites dans `assets/textures/`
- Sous-dossiers : `tiles/`, `buildings/`, `units/`, `items/`, `ui/icons/`
- Police dans `assets/fonts/`
- Atlas recommandé pour réduire les draw calls

## Placeholders

Les formes Mesh2d actuelles (square, diamond, triangle, rectangle, pentagon, circle) restent la valeur par défaut si un asset n'est pas trouvé.
