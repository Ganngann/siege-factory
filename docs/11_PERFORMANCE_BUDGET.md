# Performance Budget — Siege Factory

## Cible

| Métrique | Minimum | Recommandé | Idéal |
|---|---|---|---|
| FPS | 30 | 60 | 60+ |
| Entités simultanées | 500 | 2000 | 5000+ |
| Taille grille | 20×15 | 40×30 | 100×100 |
| Joueurs PvP | 2 | 4 | 8 |

## Hardware minimum

- CPU : 2 cœurs, 2.0 GHz
- RAM : 2 GB
- GPU : intégré (Intel HD 4000 ou équivalent)
- OS : Windows 10, Linux, macOS 10.15+

## Stratégies d'optimisation

### Rendu 2D
- Utiliser des atlas de tuiles (pas de sprites individuels) → moins de draw calls.
- `bevy_sprite2d` avec instancing pour les sprites répétés (ceintures, murs).
- Culling hors-écran (Bevy le fait automatiquement avec les `Frustum`).
- Limiter le nombre de particules/tirs visibles simultanément.

### ECS
- Les systèmes ECS de Bevy sont déjà parallélisés automatiquement.
- Utiliser `ParallelIterator` pour les opérations sur de grandes quantités d'entités.
- `Arc<[T]>` pour les données partagées en lecture seule (registres).

### Pathfinding (A*)
- A* sur grille carrée : complexité O(n) où n = taille du chemin. Pour une grille 20×15, négligeable.
- Cache de chemins : les ennemis du même type partent du même spawn → cache le chemin.
- Recalcul seulement si un building est placé/détruit sur le chemin.

### Économie
- Les ceintures et inventaires sont des opérations O(1) par tick.
- Production/Mining tick intervalle (tous les X ticks, pas à chaque frame).

## Limitations connues

- Le rendu 2D de Bevy 0.14 passe par `wgpu` (Vulkan/Metal/DX12). ~50 Mo de VRAM pour les assets de base.
- Le pathfinding A* peut devenir un goulot avec >100 ennemis simultanés. Solution : échelonner les calculs de path sur plusieurs frames.
- En PvP, la bande passante réseau est le facteur limitant avant le CPU.

## Monitoring perf

```rust
// En dev : overlay FPS + compteur d'entités
app.add_plugins((
    FrameTimeDiagnosticsPlugin,
    EntityCountDiagnosticsPlugin,
));
```

En release, ces diagnostics sont exclus via feature flags.
