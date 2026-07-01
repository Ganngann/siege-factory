# Debug & Dev Tools — Siege Factory

## Overlay FPS / Diagnostics

Utiliser `bevy::diagnostic` intégré :

```rust
app.add_plugins(FrameTimeDiagnosticsPlugin);
app.add_plugins(EntityCountDiagnosticsPlugin);
// Affiché avec un système Text overlay
```

Touche F3 pour afficher/masquer.

## Inspecteur ECS

En dev, utiliser `bevy-inspector-egui` :

```rust
// Cargo.toml — dev-dependencies
bevy-inspector-egui = "0.24"
```

```rust
// Activer avec une feature flag
#[cfg(feature = "debug")]
app.add_plugins(WorldInspectorPlugin::new());
```

Affiche : entités, composants, resources en temps réel.

## Hot reload

Bevy 0.14 supporte le hot-reload des assets :

```rust
app.add_plugins(AssetPlugin {
    watch_for_changes: true,
    ..default()
});
```

Pour le hot-reload de code, utiliser `cargo watch` :

```powershell
cargo install cargo-watch
cargo watch -x run
```

## Commands de debug (clavier)

| Touche | Action |
|---|---|
| F1 | Aide / liste des commandes |
| F3 | Toggle FPS overlay |
| F5 | Sauvegarde manuelle |
| F8 | Toggle pause |
| F9 | Avance d'un frame |
| Ctrl+Shift+R | Reset partie |
| \` | Console (future) |

## Logs

```rust
// Bevy log (trace > debug > info > warn > error)
app.add_plugins(LogPlugin {
    level: Level::DEBUG,  // INFO en prod
    filter: "wgpu=warn,siege_factory=debug".to_string(),
    ..default()
});
```

## Replay (futur)

- Enregistrer tous les inputs dans un fichier binaire (avec frame number)
- Commande `--replay <file>` pour rejouer
- Utilisé pour debug et anti-triche

## Seed debug

- `--seed <u64>` en argument pour reproduire exactement une partie
- Logguer le seed dans le fichier de save
