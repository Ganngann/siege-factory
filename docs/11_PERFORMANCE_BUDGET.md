# Performance Budget — Siege Factory

## Stratégies de scaling

### Pathfinding
- Aujourd'hui : BFS
- Demain : pathfinding hiérarchique (chunk A* + BFS local) + cache

### Rendu
- Aujourd'hui : Mesh2d (sans texture)
- Demain : atlas de tuiles, instancing, culling par chunk

### ECS
- Bevy parallélise les systèmes automatiquement
- Production/mining à intervalle (pas à chaque frame)
- `Arc<[T]>` pour les registres en lecture seule

### Économie
- Ceintures et inventaires : O(1) par tick
- Opérations locales par entité, pas de boucle globale
