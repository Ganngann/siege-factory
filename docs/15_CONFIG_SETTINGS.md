# Config & Settings — Siege Factory

## Fichier de configuration

Emplacement : `{config_dir}/siege-factory/settings.toml`

Format TOML, chargé au démarrage avec valeurs par défaut intégrées :

## Sections

```toml
[graphics]
window_width = 1280
window_height = 720
fullscreen = false
vsync = true
fps_limit = 60
scale = 1.0          # UI scaling (1.0, 1.5, 2.0)

[audio]
master_volume = 0.8
sfx_volume = 1.0
music_volume = 0.5

[gameplay]
scroll_speed = 500.0
zoom_speed = 0.1
camera_edge_scroll = true
camera_edge_threshold = 10     # pixels

[input]
# Key bindings — par défaut, surchargeables
[input.bindings]
move_up = "W"
move_down = "S"
move_left = "A"
move_right = "D"
build_mode = "B"
cancel = "Escape"
pause = "Pause"
quick_save = "F5"
quick_load = "F9"

[save]
auto_save_interval = 60         # secondes
max_saves = 20                  # nombre max de saves à garder
```

## Defaults

Intégrés dans le code via `impl Default for Settings`, chargés si le fichier n'existe pas.

## Runtime

Les settings sont stockés dans une Resource `Settings` accessible par les systèmes. Les changements sont persistés dans le fichier TOML via le menu Options.

```rust
#[derive(Resource, Serialize, Deserialize)]
struct Settings {
    graphics: GraphicsSettings,
    audio: AudioSettings,
    gameplay: GameplaySettings,
    input: InputSettings,
    save: SaveSettings,
}

impl Default for Settings {
    fn default() -> Self {
        // Valeurs par défaut
    }
}
```

## Écran Options (UI)

- Graphics : résolution, fullscreen, vsync, FPS limit
- Audio : volume master/SFX/music
- Gameplay : scroll speed, edge threshold
- Input : rebinding (clic sur touche → appuyer sur nouvelle touche → enregistré)
