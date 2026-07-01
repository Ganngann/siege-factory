# Camera & Input — Siege Factory

## Camera

Top-down 2D orthographique. Contrôles :

| Action | Input |
|---|---|
| Scroll | Bord d'écran (pixel threshold) |
| Scroll alternatif | WASD |
| Zoom | Molette |
| Zoom reset | Milieu de souris |
| Suivre sélection | Espace (centrer sur l'entité sélectionnée) |

Implémentation : `Camera2dBundle` avec `Transform` modifié par les systèmes d'input.

```rust
fn camera_scroll(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let speed = 500.0;
    let mut cam = camera.single_mut();
    if keys.pressed(KeyCode::KeyW) { cam.translation.y += speed * time.delta_seconds(); }
    if keys.pressed(KeyCode::KeyS) { cam.translation.y -= speed * time.delta_seconds(); }
    if keys.pressed(KeyCode::KeyA) { cam.translation.x -= speed * time.delta_seconds(); }
    if keys.pressed(KeyCode::KeyD) { cam.translation.x += speed * time.delta_seconds(); }
}

fn camera_zoom(
    mut camera: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    for ev in scroll_events.read() {
        let mut proj = camera.single_mut();
        proj.scale = (proj.scale - ev.y * 0.1).clamp(0.5, 3.0);
    }
}
```

## Sélection (RTS)

| Action | Input |
|---|---|
| Sélection unité | Clic gauche |
| Sélection multiple | Drag clic gauche (selection box) |
| Ajouter à sélection | Shift + clic |
| Ordre déplacement | Clic droit |
| Ordre attaque | A + clic droit |
| Build | Choisir dans build menu + clic gauche |

## Placement buildings

1. Sélectionner un building dans le menu
2. Mode placement actif : le building suit le curseur (ghost, vert/rouge selon validité)
3. Clic gauche : placer
4. Clic droit ou Échap : annuler le placement

## Validation placement

Un placement est valide si :
- La zone est libre (pas d'autre building)
- Le terrain est constructible (pas sur gisement, pas sur spawner)
- Le joueur a assez de ressources
- La zone est dans la carte

## Raccourcis clavier

| Touche | Action |
|---|---|
| Échap | Annuler / Fermer menu |
| Espace | Centrer sur sélection |
| Q / W / E / R | Build shortcuts (plus tard) |
| 1-4 | Sélection groups (plus tard) |
| Ctrl+A | Sélectionner toutes les unités |
| Ctrl+Z | Undo (si supporté) |
| Pause | Pause |
| Tab | Cycle building menu |
