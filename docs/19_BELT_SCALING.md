# Scaling Belt — Siege Factory

## Problème

Le stockage individuel par item (`Vec<Option<ItemOnBelt>>` par tuile) ne passe pas à l'échelle :
- 1 item ≈ 32 bytes → 1 milliard d'items = 32 GB
- Itérer chaque item à 20 Hz = 100 milliards ops/sec

## Solution : ItemBlock + BeltSegment

### Concept

Au lieu de stocker les items individuellement, on stocke des **blocs d'items** contigus homogènes sur des **segments** de belt.

Un **segment** = suite continue de belts de même direction et vitesse, sans bifurcation.

```
BeltSegment (direction Est, 10 tuiles × 5 slots = 50 slots)
  └── ItemBlock { pattern: [fer], repetition: 50 }
       → 50 items stockés en ~40 bytes au lieu de 1600 bytes
```

### Gains

| Aspect | Avant | Après |
|---|---|---|
| Mémoire | O(items) | O(blocs) — 1M fer → 40 KB |
| CPU | O(items/tick) | O(blocs/tick) — 200k blocs → 4M ops/sec |

Le pire cas (aucune répétition) dégénère en O(1) bloc par item, mais la production en usine génère massivement des files homogènes.
