# Build & CI — Siege Factory

## Build local

```powershell
# Dev (rapide, debug)
cargo build

# Release (optimisé, LTO)
cargo build --release

# Web (WASM)
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown
```

## Serveur headless (pas de rendu)

```rust
// src/bin/server.rs — plus tard
fn run_headless() {
    App::new()
        .add_plugins(CorePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(EconomyPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(CombatPlugin)
        // Pas de DefaultPlugins, pas de UI
        .run();
}
```

## Linters

```powershell
cargo clippy                  # Linting
cargo clippy -- -D warnings   # Bloque si warnings
cargo fmt --check             # Formatage
```

## GitHub Actions (CI)

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo clippy -- -D warnings
      - run: cargo test
  build-wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: rustup target add wasm32-unknown-unknown
      - run: cargo build --target wasm32-unknown-unknown
```

## Release (Steam)

```powershell
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
```

Emballage avec `cargo-bundle` ou manuel. Intégration Steam via `steamworks-rs`.

## Version WASM en ligne

Utiliser `wasm-bindgen` et `wasm-opt` pour servir le jeu via un simple serveur HTTP (ou GitHub Pages).
