# Config & Settings — Siege Factory

## Architecture

Les settings sont stockés dans une Resource ECS `Settings`, persistée en TOML.

```
fichier: {config_dir}/siege-factory/settings.toml
format : TOML
defauts: impl Default for Settings (intégré dans le code)
```

### Sections

- Graphics : résolution, fullscreen, vsync, FPS limit, UI scale
- Audio : volumes master / SFX / music
- Gameplay : scroll speed, zoom speed, edge scroll
- Input : key bindings (rebinding via menu)
- Save : auto_save_interval, max_saves

### Runtime

Les changements sont effectués via le menu Options et persistés dans le fichier TOML. Les systèmes lisent la Resource `Settings` pour leurs comportements.
