# Multiplayer — Siege Factory

## Architecture cible

P2P avec simulation déterministe locale. Chaque pair simule la partie complète. Les inputs sont échangés en P2P via QUIC (quinn). Un serveur web minimal sert de notaire pour le matchmaking, ELO, et preuve anti-triche.

## Principe

```
[Pair A] ── inputs signés ──→ [Pair B]
[Pair B] ── inputs signés ──→ [Pair A]
          ↕ échange asynchrone
               ↓
    Chacun simule localement la partie
    L'état converge car seed déterministe + mêmes inputs
               ↓
    [Web Notaire] ← hashs d'état signés périodiquement
```

## Éléments à prépare r dès maintenant

### 1. Command Pattern

Toute action "importante" passe par un Event :

```rust
// Inputs deviendront des messages réseau
#[derive(Event)]
struct BuildOrder {
    player_id: NetworkId,
    building: BuildingKind,
    position: TilePosition,
    frame: u64,           // horloge logique
}

#[derive(Event)]
struct MoveOrder {
    player_id: NetworkId,
    unit_entity: NetworkId,
    target: TilePosition,
    frame: u64,
}
```

### 2. Simulation déterministe

```rust
#[derive(Resource)]
struct GameSeed(u64);

#[derive(Resource)]
struct FrameNumber(u64);
```

- Tout le RNG utilise `GameSeed` comme seed source. Même seed = même génération de carte, même comportement ennemi.
- Les systèmes sont ordonnés via `SystemSet` — l'ordre d'exécution est identique sur toutes les machines.
- `FrameNumber` s'incrémente à chaque tick logique.

### 3. NetworkId

Les entités "importantes" reçoivent un `NetworkId` :

```rust
#[derive(Component)]
struct NetworkId(u64);
```

Permet de référencer des entités entre deux machines. Les entités éphémères (projectiles, particules) n'en ont pas besoin.

### 4. Horloge logique

Chaque tick de simulation a un numéro de frame unique partagé par tous les pairs. Les inputs sont horodatés par ce frame.

### Anti-triche

| Mécanisme | Description | Coût |
|---|---|---|
| **Simulation déterministe** | Mêmes inputs → même état partout | Fondation |
| **Inputs signés** | Chaque ordre signé avec keypair | Crypto légère |
| **Hash d'état** | Comparaison périodique de l'état économique | Détection écart |
| **Replay** | Log complet des inputs → rejouable | Preuve post-match |
| **Notaire** | Serveur web stocke les hashs signés ($3-5/mois ou Cloudflare gratuit) | Auth + ELO |

### Ce qu'on ne fait pas maintenant

- `quinn` / `libp2p` — implémenté quand le PvP est prêt
- Crypto des inputs — quand le classé arrive
- Anti-triche temps réel — quand y'a des matchs ranked

### Stack cible (plus tard)

- **Transport** : `quinn` (QUIC, UDP fiable, P2P)
- **Synchronisation** : proche de `bevy_replicon` mais adapté P2P
- **Notaire** : `actix-web` ou `axum` minimal + PostgreSQL ou SQLite
- **Matchmaking** : même serveur, endpoints REST simples

## Notes

Le but n'est pas d'implémenter le multi maintenant, mais de s'assurer que l'architecture ECS + data-driven + déterministe le permet sans réécriture.
