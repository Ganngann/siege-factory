# Siege Factory — Agent Guide

## Stack

Bevy 0.14, Rust 1.96+, ECS (Entity Component System)

## Commandes

- `cargo run` — lancer le jeu
- `cargo test` — tests unitaires + intégration
- `cargo clippy` — lint
- `cargo build --target wasm32-unknown-unknown` — build web

## Architecture

Voir `docs/2_ARCHITECTURE.md` pour modules, plugins, SystemSets.
Règle : 1 système = 1 responsabilité.

## Data-driven

Définitions dans `data/*.toml`, chargées via registres.
Voir `docs/3_DATA_DESIGN.md`.

## Conventions

- Events pour actions importantes (BuildOrderEvent, SpawnWaveEvent)
- Fonctions pures `compute_*` extraites et testées
- Pas de logique dans le rendu
- Components sérialisables pour save/load
- Voir `docs/6_CODING_CONVENTIONS.md`

## Multi

Simulation déterministe, seed commune, NetworkId.
Voir `docs/4_MULTIPLAYER.md` — ne pas implémenter maintenant.

## Tests

- `cargo test` pour le CI
- `proptest` pour les invariants
- Tests headless (App sans rendu)
- Voir `docs/12_TESTING_STRATEGY.md`
