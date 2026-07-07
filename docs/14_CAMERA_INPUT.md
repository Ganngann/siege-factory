# Camera & Input — Siege Factory

## Camera

Top-down 2D orthographique via `Camera2dBundle`. La caméra suit le joueur automatiquement.

- Zoom : molette

## Input → Action

Les actions utilisateur passent par des **Events** :
- Clic menu → `BuildOrderEvent`
- Déplacement, minage, interaction → touches
- Les systèmes de logique consomment ces events
