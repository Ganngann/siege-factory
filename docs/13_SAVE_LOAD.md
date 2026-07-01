# Save & Load — Siege Factory

## Format

Sérialisation binaire via `bincode` ou `postcard`. L'état complet du monde ECS est sérialisé :

- Resources : `GameState`, `FrameNumber`, `GameSeed`
- Ressources économiques : `Inventory` de chaque entité
- Buildings : position, type, HP, inventaire interne
- Ennemis : type, position, HP, path actuel
- Vagues : état actuel, timer, prochaine vague

Pas de sérialisation des entités éphémères (projectiles, particules).

## Structure

```rust
#[derive(Serialize, Deserialize)]
struct SaveData {
    version: u32,                          // Format version
    timestamp: u64,                        // Save date
    seed: u64,                             // Game seed
    frame: u64,                            // Logical frame
    state: GameState,                      // Current game state
    entities: Vec<SerializedEntity>,       // All persistent entities
    next_wave_index: usize,
    wave_timer: f32,
}
```

## Emplacement

- Windows : `%APPDATA%/siege-factory/saves/`
- Linux : `~/.local/share/siege-factory/saves/`
- macOS : `~/Library/Application Support/siege-factory/saves/`

Utiliser `dirs::data_dir()` pour le chemin portable.

## Noms de saves

- Auto-save : `autosave.{timestamp}.sav`
- Save manuelle : `save_{index}.sav`
- Quick save : `quicksave.sav`

## Fréquence auto-save

- Toutes les 60 secondes
- Au début de chaque nouvelle vague
- Avant une action critique (début combat de boss)

## Load

- Au démarrage, bouton "Continuer" si un save existe
- Écran de sélection de save
- Le load restore tout l'état ECS et continue la simulation

## Sérialisation ECS

Bevy n'a pas de sérialisation ECS intégrée. On utilise `bevy-serialize` ou une approche manuelle :

```rust
// Parcourir les Query et collecter les composants à sauvegarder
// Les entités non-persistantes (projectiles) sont ignorées
fn collect_save_data(
    buildings: Query<(Entity, &Building, &TilePosition, &HP, &Inventory)>,
    enemies: Query<(Entity, &Enemy, &Transform, &HP)>,
) -> Vec<SerializedEntity> {
    // ...
}
```

## Compatibilité

- `version` dans le header permet la migration si le format change.
- Version majeure : format break, pas de compatibilité ascendante.
- Version mineure : champs optionnels ajoutés, load possible.
