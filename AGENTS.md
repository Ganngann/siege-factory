# Siege Factory — Agent Guide

## Stack

Bevy 0.19, Rust 1.96, ECS (Entity Component System)

## Commands

### Auto-autorisés (lecture seule / safe)
- `ls`, `cd`, `cat`, `rg`, `fd`, `Test-Path`, `Select-String`
- `git status`, `git diff`, `git log`
- `cargo check`, `cargo run`, `cargo test`, `cargo fmt`, `cargo doc`

### Sur demande explicite
- `cargo clippy`
- `cargo build --target wasm32-unknown-unknown`
- `.\build_wasm.ps1` — WASM build + wasm-bindgen + copy index.html to web/

## Vision (destination)

Factorio-like: infinite map, multiplayer, deep tech tree, branching recipes, N resources.
Current tower-defense mode is a **scaffold** to build the tech base incrementally.

## Architecture

Modules: `core` (GameState/UI), `map` (grid/tiles), `economy` (resources/buildings), `enemy` (waves/pathfinding), `unit` (Soldier/Worker), `combat` (projectiles/damage), `rendering` (shapes/HP bars).
Rule: 1 system = 1 responsibility.

## Data-driven

Definitions in `data/*.toml`, loaded via registries.
See `docs/3_DATA_DESIGN.md`.

## Conventions

- Events for important actions (BuildOrderEvent, SpawnUnitEvent, SpawnWaveEvent)
- Pure functions `compute_*` extracted and tested
- No logic in rendering
- Components prepared for save/load (future)
- See `docs/6_CODING_CONVENTIONS.md`

## Multi

Simulation deterministic, common seed, NetworkId.
See `docs/4_MULTIPLAYER.md` — planned, not before solo base stable.

## Tests

- `cargo test` for CI
- `proptest` for invariants
- Headless tests (App without rendering)
- See `docs/12_TESTING_STRATEGY.md`

## Modules

- `core` — GameState (Menu/Playing/GameOver), transitions, input, settings, tooltips, main menu
- `map` — TileGrid 20×15, Tile/TileType/TilePosition, fixed deposits + procedural generation
- `economy` — ResourceId/Inventory, BuildingDef/Registry, HQ/OreDeposit/Miner/Assembler/Belt/Wall/Turret/Storage/Splitter/Sorter, dynamic menu tree, production ticks, ore/ammo/energy UI
- `enemy` — Enemy/Health components, WaveState, BFS pathfinding, spawn/move/combat, turret auto-shoot, GameOver screen (waves survived)
- `unit` — Soldier/Worker components, spawn via menu
- `combat` — Projectiles, homing, damage systems
- `rendering` — Mesh2d shapes, HP bars, tile highlight, belt item rendering
