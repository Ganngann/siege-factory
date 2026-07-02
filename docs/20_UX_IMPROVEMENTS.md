# UX Improvements — Plan détaillé

> Remplace `19_DATA_DRIVEN_REFACTOR.md` (livré).
> Projet passé à Bevy 0.19. Document rédigé le 2026-07-02.

---

## Problèmes identifiés

1. **Escape conflict** — Annule le build mode ET forfeit la partie en même frame
2. **Pas de clic droit** — Aucun moyen standard de déselectionner
3. **Silent failures** — Placement/spawn raté = rien ne se passe
4. **Pas de tile highlight** — Aucun indicateur de la tuile survolée
5. **Pas de HP bars** — Impossible d'évaluer les dégâts
6. **Pas de wave announcement** — La vague change sans indication visuelle
7. **Pas de tooltips** — Les boutons n'expliquent pas leur rôle
8. **Pas de caméra** — Pas de pan/zoom

---

## Dépendances ajoutées

| Crate | Version | Utilité |
|-------|---------|---------|
| `bevy_pancam` | ^0.19 | Caméra pan + zoom (2D) |
| `leafwing_input_manager` | ^0.19 | Actions nommées, rebind futur |
| `bevy_tweening` | ^0.19 | Animations fluides (HP, toasts) |
| `bevy_fog_of_war` | ^0.19 | Brouillard de guerre (M6.5) |
| `bevy_kira_audio` | ^0.19 | Audio (M5.5, plus tard) |

Ajout dans `Cargo.toml`, plugins dans `lib.rs`.

---

## 1. `bevy_pancam` — Caméra pan/zoom

**Fichier** : `src/map/systems.rs:20-27`

Remplacer `Camera2dBundle::default()` fixe par :

```rust
use bevy_pancam::PanCam;

commands.spawn((
    Camera2dBundle {
        transform: Transform::from_xyz(
            cfg.width as f32 * cfg.tile_size / 2.0,
            cfg.height as f32 * cfg.tile_size / 2.0,
            100.0,
        ),
        ..default()
    },
    PanCam {
        speed: 500.0,
        min_scale: 0.3,
        max_scale: 3.0,
        min_x: 0.0,
        max_x: cfg.width as f32 * cfg.tile_size,
        min_y: 0.0,
        max_y: cfg.height as f32 * cfg.tile_size,
        ..default()
    },
));
```

**Problème connu** : `cursor_tile` dans `placement.rs` utilise `viewport_to_world_2d` qui tient déjà compte de la position de la caméra. Fonctionne sans modification.

---

## 2. `leafwing_input_manager` — Input mapping

**Fichier** : `src/core/actions.rs` (nouveau)

Définit les actions nommées et leurs bindings par défaut. Les systèmes utilisent `ActionState<GameAction>` au lieu de `Res<ButtonInput<KeyCode>>`.

```rust
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
pub enum GameAction {
    BuildWall, BuildMiner, BuildAssembler, BuildBelt, BuildTurret,
    SpawnSoldier, SpawnWorker,
    Cancel, Rotate, Restart,
}
```

**Mapping action → building ID** : fonction utilitaire dans `placement.rs` :

```rust
fn action_to_building_id(action: &GameAction) -> Option<&'static str> {
    match action {
        GameAction::BuildWall => Some("wall"),
        GameAction::BuildMiner => Some("miner"),
        GameAction::BuildAssembler => Some("assembler"),
        GameAction::BuildBelt => Some("belt"),
        GameAction::BuildTurret => Some("turret"),
        _ => None,
    }
}
```

**Migration** : remplacer tous les `keys.just_pressed(KeyCode::Digit1)` par `action_state.just_pressed(GameAction::BuildWall)` dans :
- `economy/placement.rs` — `build_mode_input` (touches 1-5, R, Escape)
- `unit/mod.rs` — `spawn_unit_input` (touches 6-7)
- `core/schedule.rs` — `game_state_transition` (Escape, R)

---

## 3. Fix Escape conflict

**Avant** : `build_mode_input` (placement.rs) ET `game_state_transition` (schedule.rs) réagissent à `just_pressed(Escape)` dans le même frame.

**Après** : `build_mode_input` ne gère plus Escape. `game_state_transition` lit `BuildMode` :

```rust
fn game_state_transition(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    action_state: Res<ActionState<GameAction>>,
    build_mode: Option<Res<BuildMode>>,
) {
    if action_state.just_pressed(GameAction::Cancel) {
        let mode_active = build_mode.as_ref().map(|m| m.0.is_some()).unwrap_or(false);
        if mode_active {
            if let Some(mut bm) = build_mode {
                bm.0 = None;
            }
        } else {
            next_state.set(GameState::GameOver);
        }
    }
    if action_state.just_pressed(GameAction::Restart) && *state.get() == GameState::GameOver {
        next_state.set(GameState::Playing);
    }
}
```

Dans `placement.rs`, supprimer le bloc Escape (lignes 85-87).

---

## 4. Right-click cancel

**Fichier** : `src/economy/placement.rs`

Dans `build_mode_input`, ajouter :

```rust
if buttons.just_pressed(MouseButton::Right) {
    build_mode.0 = None;
}
```

Ajouter `buttons: Res<ButtonInput<MouseButton>>` aux paramètres.

---

## 5. Toast feedback

### Architecture

```
ToastEvent(String) → toast_system → spawn TextBundle → despawn après 3s
```

### Fichiers

- **`src/core/toast.rs`** (nouveau) : `ToastMessage { timer: f32 }` component + `toast_system`
- **`src/events.rs`** : ajouter `#[derive(Event)] pub struct ToastEvent(pub String);`

### Système toast

```rust
pub fn toast_system(
    mut commands: Commands,
    mut events: EventReader<ToastEvent>,
    time: Res<Time>,
    toasts: Query<(Entity, &mut ToastMessage)>,
) {
    for ev in events.read() {
        commands.spawn((
            ToastMessage { timer: 3.0 },
            TextBundle::from_section(ev.0, TextStyle { font_size: 16.0, color: Color::srgb(1.0, 0.85, 0.3), ..default() }),
        ));
    }
    for (entity, mut msg) in toasts.iter_mut() {
        msg.timer -= time.delta_seconds();
        if msg.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### Sites d'émission

Dans `placement.rs` — `handle_build_click`, émettre `ToastEvent` avant chaque `return` silencieux :
- "Not enough ore"
- "Tile occupied"
- "No ore deposit here"

Dans `unit/mod.rs` — `spawn_unit_input`, avant le `return` quand pas assez d'ore.

---

## 6. HoveredTile + Tile highlight

### HoveredTile resource

**Fichier** : `src/map/components.rs`

```rust
#[derive(Resource, Default)]
pub struct HoveredTile(pub Option<TilePosition>);
```

Extraire `cursor_tile` de `placement.rs:15-30` vers `map/components.rs` comme fonction publique `cursor_to_tile`.

Nouveau système `update_hovered_tile` qui remplit `HoveredTile` à chaque frame :

```rust
pub fn update_hovered_tile(
    mut hovered: ResMut<HoveredTile>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
) {
    hovered.0 = cursor_to_tile(&windows, &camera, &cfg);
}
```

Migrer `placement.rs` : remplacer les appels à `cursor_tile` par la lecture de `hovered.0`.

### Tile highlight

**Fichier** : `src/rendering.rs`

Nouveau système qui affiche un carré semi-transparent sur la tuile survolée (seulement quand `BuildMode` est `None`) :

```rust
pub fn tile_highlight(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    hovered: Res<HoveredTile>,
    cfg: Res<MapConfig>,
    existing: Query<Entity, With<TileHighlight>>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }
    if build_mode.0.is_some() { return; }
    let Some(pos) = hovered.0 else { return; }
    commands.spawn((
        TileHighlight,
        ColorMesh2dBundle {
            mesh: Mesh2dHandle(shapes.square.clone()),
            material: materials.add(ColorMaterial::from_color(Color::srgba(1.0, 1.0, 1.0, 0.15))),
            transform: Transform::from_xyz(pos.x as f32 * cfg.tile_size, pos.y as f32 * cfg.tile_size, 0.5),
            ..default()
        },
    ));
}
```

---

## 7. HP bars

**Fichiers** : `src/economy/components.rs` (component) + `src/rendering.rs` (systèmes)

```rust
#[derive(Component)]
pub struct HpBarChild;
```

Deux systèmes :
- `ensure_hp_bars` : pour chaque entité avec `Health` sans `HpBarChild`, spawn un enfant `Sprite` (rectangle 24×3) au-dessus
- `update_hp_bars` : ajuste `custom_size.x` à `24.0 * (current / max)` et change la couleur (vert > 60%, jaune > 30%, rouge < 30%)

---

## 8. Wave announcement

**Fichier** : `src/enemy/systems.rs`

Resource `LastWave(u32)`. Système `wave_announcement` qui détecte `wave.wave != last_wave.0` et spawn un texte temporaire "Wave X!" avec `ToastMessage { timer: 2.0 }`.

---

## 9. Tooltips

**Fichier** : `data/buildings.toml` + `src/core/tooltip.rs` (nouveau)

Ajouter `description` à chaque building dans le TOML. Le système `tooltip_ui` lit `TooltipText` resource et affiche un texte flottant près du curseur.

---

## 10. `bevy_tweening` — Animations

Utiliser des `Tween` pour les transitions fluides :
- HP bars : `Tween<Transform>` sur l'échelle X (durée 200ms, easing QuadraticOut)
- Wave announcement : `Tween<Text>` sur l'opacité (fondu)

---

## 11. `bevy_fog_of_war` — M6.5

Plugin ajouté mais systèmes inactifs pour l'instant.

---

## 12. `bevy_kira_audio` — M5.5

Plugin ajouté mais inactif. Prévoir les events `PlaySfxEvent(SfxKind)` pour plus tard.

---

## Fichiers impactés

| Fichier | Action |
|---------|--------|
| `Cargo.toml` | +5 dépendances |
| `src/lib.rs` | +5 plugins |
| `src/core/actions.rs` | **Nouveau** |
| `src/core/toast.rs` | **Nouveau** |
| `src/core/tooltip.rs` | **Nouveau** |
| `src/core/mod.rs` | +3 `pub mod` |
| `src/core/schedule.rs` | Modifier `game_state_transition` |
| `src/economy/components.rs` | +`HpBarChild` |
| `src/economy/building.rs` | +`description: String` |
| `src/economy/placement.rs` | -Escape, +Right-click, +Toast, +leafwing |
| `src/economy/build_bar.rs` | +Tooltips |
| `src/map/components.rs` | +`HoveredTile`, +`cursor_to_tile` |
| `src/map/systems.rs` | +`PanCam`, +`update_hovered_tile` |
| `src/unit/mod.rs` | +Toast, +leafwing |
| `src/rendering.rs` | +`tile_highlight`, +hp bars |
| `src/enemy/components.rs` | +`LastWave` |
| `src/enemy/systems.rs` | +`wave_announcement` |
| `src/events.rs` | +`ToastEvent` |

## Ordre d'exécution

```
1. docs/20_UX_IMPROVEMENTS.md     ← ce fichier
2. Cargo.toml                     dépendances
3. lib.rs + core/mod.rs           plugins + modules
4. core/actions.rs                leafwing GameAction
5. map/components.rs              HoveredTile + cursor_to_tile
6. core/schedule.rs               fix Escape
7. core/toast.rs                  ToastEvent + système
8. core/tooltip.rs                TooltipText + système
9. events.rs                      ToastEvent
10. economy/placement.rs          -Escape, +Right-click, +Toast, +leafwing
11. economy/build_bar.rs          +Tooltips
12. economy/building.rs           +description
13. map/systems.rs                +PanCam, +update_hovered_tile
14. unit/mod.rs                   +Toast, +leafwing
15. rendering.rs                  +tile_highlight, +hp bars
16. data/buildings.toml           +descriptions
17. enemy/components.rs           +LastWave
18. enemy/systems.rs              +wave_announcement
19. economy/mod.rs                +ToastEvent registration
20. cargo check
```
