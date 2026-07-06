# Siege Factory

Mélange RTS + automation + tower defense (2D top-down).
Construisez votre usine, automatisez la production, repoussez des vagues d'ennemis. Évolution vers le PvP.

## Stack

- **Moteur** : Bevy 0.14 (Rust, ECS)
- **Langage** : Rust 1.96+
- **Build** : `cargo build`
- **Tests** : `cargo test`
- **Web** : `cargo build --target wasm32-unknown-unknown`

## Lancer le jeu

```powershell
cd siege-factory
cargo run
```

## Lancer les tests

```powershell
cargo test                    # Tests unitaires + intégration
cargo test --release          # Tests longs + proptest
cargo clippy                  # Linting
```

## Structure du projet

```
src/           → Code source (ECS)
data/          → Définitions de jeu (TOML)
assets/        → Sprites, tuiles, sons
docs/          → Documentation
```

## Roadmap

| Milestone | Contenu | Statut |
|---|---|---|---|
| M1 | Squelette, grille, états, tests | ✅ |
| M2 | Économie, ressources, inventaire | ✅ |
| M3 | Constructions, ceintures, craft, Découverte + Archive | ✅ |
| M4 | Ennemis, combat, vagues | |
| M5 | Polissage, build web, équilibrage | |
| M6 | Unités RTS, tech, fog of war | |
| M7 | Multijoueur P2P | |
| M8 | Release Steam | |

## Licence

Projet personnel — tous droits réservés.
