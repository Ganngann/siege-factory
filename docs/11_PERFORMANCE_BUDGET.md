# Performance Budget — Siege Factory

## Cible actuelle (scaffold TD 20×15)

| Métrique | Minimum | Actuel |
|---|---|---|
| FPS | 30 | 60+ |
| Entités simultanées | 500 | <200 |
| Taille grille | 20×15 | 20×15 |
| Joueurs | 1 | 1 |

## Cible destination (Factorio-like)

| Métrique | Minimum | Idéal |
|---|---|---|
| FPS | 30 | 60 |
| Entités simultanées | 5000 | 20000+ |
| Taille carte | 100×100 chunks | Infini |
| Joueurs PvP | 4 | 8 |

## Stratégies pour y arriver

### Pathfinding
- Aujourd'hui : BFS sur 300 tuiles
- Demain : pathfinding hiérarchique (chunk A* + BFS local) + cache de chemins

### Rendu
- Aujourd'hui : Mesh2d (sans texture)
- Demain : atlas de tuiles, instancing, culling par chunk

### ECS
- Bevy parallélise les systèmes automatiquement
- Production/mining à intervalle (pas à chaque frame)
- `Arc<[T]>` pour les registres en lecture seule

### Économie
- Ceintures et inventaires : O(1) par tick
- Pas de boucle sur toute la grille (operations locales par entité)

## Limitations connues

- BFS sur 300 tuiles : négligeable aujourd'hui. Sur une grande carte, il faudra un cache et un pathfinding hiérarchique.
- Le rendu Mesh2d de Bevy passe par wgpu (Vulkan/Metal/DX12). ~50 Mo VRAM pour les assets de base.
- En PvP, la bande passante réseau sera le facteur limitant.

## Monitoring

```rust
app.add_plugins((
    FrameTimeDiagnosticsPlugin,
    EntityCountDiagnosticsPlugin,
));
```
