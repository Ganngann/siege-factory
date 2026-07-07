# Multiplayer — Siege Factory

## Architecture cible

P2P avec simulation déterministe locale. Chaque pair simule la partie complète. Les inputs sont échangés en pair-à-pair via un transport fiable (QUIC).

### Principe

```
[Pair A] ── inputs ──→ [Pair B]
[Pair B] ── inputs ──→ [Pair A]
           ↕ échange asynchrone
                ↓
    Chacun simule localement la partie
    L'état converge car seed déterministe + mêmes inputs
```

### Prérequis architecture (en place)

- Toute action "importante" passe par un **Event** (Command Pattern)
- Le RNG utilise une **seed commune** déterministe
- Les systèmes sont ordonnés via `SystemSet` — ordre d'exécution identique
- Les entités "importantes" reçoivent un **NetworkId** pour référence cross-machine
- **FrameNumber** incrémenté à chaque tick logique pour horodater les inputs

Ces éléments sont préparés dès maintenant pour permettre le multi sans réécriture.
