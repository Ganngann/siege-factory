# Save & Load — Siege Factory

## Architecture

Le save/load sérialise l'état persistant du monde ECS au format sérialisé (actuellement RON).

### Principe

- **Ce qui persiste** : entités durables (bâtiments, unités, ennemis, inventaires, état du monde)
- **Ce qui ne persiste pas** : entités éphémères (projectiles, particules, effets visuels)

### Flux

1. Déclencheur (menu, touche) → événement de sauvegarde
2. Itération des queries ECS → structure de données sérialisable
3. Sérialisation → écriture sur disque
4. Au load : lecture → désérialisation → spawn des entités par type

### Emplacement

`%APPDATA%/siege-factory/saves/`

## Bug connu

- Building 2×2 décalé d'1/2 case au load
