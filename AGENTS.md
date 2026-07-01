# Siege Factory — Agent Guide

## Stack

Bevy 0.14, Rust 1.96+, ECS (Entity Component System)

## Commandes

- `cargo run` — lancer le jeu
- `cargo test` — tests unitaires + intégration
- `cargo clippy` — lint
- `cargo build --target wasm32-unknown-unknown` — build web (WASM)
- `.\build_wasm.ps1` — build web + wasm-bindgen + copie index.html dans web/

## Architecture

Modules: `core` (GameState/UI), `map` (grille/tuiles), `economy` (ressources/bâtiments), `enemy` (vagues/ennemis/combat), `unit` (Soldier/Worker).
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

- `cargo test` pour le CI (22 tests — 21 unit + 1 integration)
- `proptest` pour les invariants
- Tests headless (App sans rendu)
- Voir `docs/12_TESTING_STRATEGY.md`

## Modules

- `core` — GameState (Loading/Playing/GameOver), transitions, Loading UI, `schedule.rs` tests
- `map` — TileGrid 20×15, Tile/TileType/TilePosition, rendu checkerboard
- `economy` — ResourceId/Inventory, BuildingDef/Registry, HQ/OreDeposit/Miner/Assembler/Belt/Wall/Turret, build palette (keys 1-5), production ticks, ore/ammo/energy UI
- `enemy` — Enemy/Health components, WaveState, BFS pathfinding, spawn/move/combat, turret auto-shoot, GameOver screen (waves survived)
- `unit` — Soldier/Worker components, spawn unit input (key 6/7), soldier auto-attack
