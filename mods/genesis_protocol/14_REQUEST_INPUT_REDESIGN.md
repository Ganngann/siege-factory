# Request 14 — Refonte complète du système d'interaction

## Contexte

Le système d'interaction actuel souffre de conflits et d'incohérences :
- `E` utilisé à la fois pour miner (hold) ET interagir avec la capsule (press) → impossible d'être près des deux sans déclencher les deux actions
- Clic gauche limité par une distance de 3 tuiles (`inspect_range_tiles`) alors que les jeux du genre permettent un clic illimité pour la consultation
- Aucun tooltip au survol → le joueur ne sait pas ce qu'il va voir en cliquant
- `E` ne fait rien près des bâtiments normaux → vide交互
- `T` et `P` pour le transfert d'items sont remplacés par un glisser-déposer (déjà convenu)

---

## Analyse des standards du genre

| Jeu | Clic gauche bâtiment | E / action | Tooltip survol | Distance clic |
|-----|---------------------|------------|----------------|---------------|
| **Factorio** | Ouvre panneau (illimité) | F pour prendre items | Oui (nom + état) | Illimitée |
| **Satisfactory** | Ouvre panneau (proximité 3D) | E interagit | Non (jeu 3D) | Proximité jeu 3D |
| **DSP** | Ouvre panneau (illimité) | F récolter | Oui (ressources) | Illimitée |
| **Mindustry** | Ouvre panneau (illimité) | — | Oui (infos rapides) | Illimitée |

**Constat :** Le clic gauche pour ouvrir un panneau est illimité dans tous les jeux top-down du genre. La limitation de distance n'existe que dans les jeux 3D (Satisfactory) où le clic gauche est aussi l'action primaire.

---

## Proposition de refonte

### 1. Clic gauche → panneau illimité

**Fichier :** `src/economy/inspect/interaction.rs` (`building_inspect_click`)

**Problème :** Lignes 81-89, la vérification de proximité bloque l'ouverture du panneau si le joueur est à plus de 3 tuiles.

**Fix :** Déplacer la vérification de proximité **après** l'ouverture du panneau, ou la supprimer. Le panneau doit s'ouvrir à n'importe quelle distance visible à l'écran.

```rust
// AVANT — bloqué par distance
let in_proximity = footprint.iter().any(|(tx, ty)| {
    let tile_center = tile_to_world(*tx, *ty, cfg.tile_size);
    let (wx, wy) = (tile_center.x, tile_center.y);
    let dx = player_pos.0.x - wx;
    let dy = player_pos.0.y - wy;
    dx * dx + dy * dy <= interact_range_sq
});
if !in_proximity {
    return;
}

// APRÈS — pas de vérification de distance pour le clic
// (la distance sera vérifiée seulement pour les actions E, T, P)
```

**Comportement attendu :**
- Le joueur peut cliquer sur n'importe quel bâtiment visible → le panneau s'ouvre
- Le panneau peut afficher les infos en lecture seule (état, recettes en cours) mais désactiver les actions nécessitant la proximité (changer recette, upgrade, transfert)

### 2. `E` devient contextuel et prioritaire

**Fichier :** `src/economy/tiered_structure.rs` (`structure_interact`) + `src/economy/player.rs` (`player_mine`)

**Problème actuel :**
- `player_mine` utilise `keys.pressed(KeyCode::KeyE)` — fires chaque frame tant que E est maintenu
- `structure_interact` utilise `keys.just_pressed(KeyCode::KeyE)` — fires une fois au pressage
- Les deux peuvent se déclencher en même temps si la capsule ET un dépôt sont à portée

**Fix — Système de priorité centralisé :**

Créer un système `contextual_interact` dans un nouveau fichier `src/player/interact.rs` (ou dans `src/economy/interact.rs`) qui :
1. Vérifie `keys.just_pressed(KeyCode::KeyE)` pour l'action "press" ET `keys.pressed(KeyCode::KeyE)` pour l'action "hold"
2. Cherche une cible prioritaire autour du joueur (cases adjacentes + 1 de range)
3. Applique l'action selon la priorité :

```rust
// Ordre de priorité pour l'action E
// Press (just_pressed) :
//   1. Capsule/structure à tiers → donner items requis (appel vers tiered_structure)
//   2. Data pad → lire
// Hold (pressed) :
//   3. Dépôt minable → miner (si aucune cible priority plus haute n'est à portée)
//
// Si aucune cible à portée → ne rien faire (toast "rien à faire ici")
```

**Code conceptuel :**

```rust
pub fn contextual_interact(
    keys: Res<ButtonInput<KeyCode>>,
    player_q: Query<(&TilePosition, &Inventory), With<Player>>,
    spatial: Res<SpatialRegistry>,
    tier_q: Query<&CurrentTier, With<Capsule>>,
    building_q: Query<&Building>,
    registry: Res<BuildingRegistry>,
    deposit_q: Query<&ResourceDeposit>,
    mut commands: Commands,
    mut archive: ResMut<GlobalArchive>,
    mut toasts: ResMut<ToastQueue>,
    mut mining_timer: ResMut<MiningTimer>,
    time: Res<Time>,
    cfg: Res<MapConfig>,
    tool_registry: Res<ToolRegistry>,
) {
    let Ok((player_tile, player_inv)) = player_q.single() else { return; };
    let check_tiles = [(0,0), (1,0), (-1,0), (0,1), (0,-1)];
    let mut entities_nearby: Vec<(Entity, i32)> = Vec::new();

    // Collecter toutes les entités à portée
    for &(dx, dy) in &check_tiles {
        let tx = player_tile.x + dx;
        let ty = player_tile.y + dy;
        if let Some(entity) = spatial.at(tx, ty) {
            entities_nearby.push((entity, 0));
        }
    }

    // --- PRESS (just_pressed) ---
    if keys.just_pressed(KeyCode::KeyE) {
        // Priorité 1 : Capsule / structure à tiers
        for (entity, _) in &entities_nearby {
            if let Ok(building) = building_q.get(*entity) {
                if let Some(def) = registry.get(&building.kind) {
                    if !def.tiers.is_empty() && tier_q.contains(*entity) {
                        // Exécuter l'interaction capsule (donner les items)
                        structure_interact_at(entity, ...); // extrait de tiered_structure
                        return;
                    }
                }
            }
        }

        // Priorité 2 : Data pad
        // ... à implémenter
    }

    // --- HOLD (pressed) ---
    if keys.pressed(KeyCode::KeyE) {
        // Priorité 3 : Dépôt minable (seulement si aucune capsule n'a été trouvée)
        if !entities_nearby.iter().any(|(e, _)| has_capsule(*e, ...)) {
            for (entity, _) in &entities_nearby {
                if let Ok(deposit) = deposit_q.get(*entity) {
                    if deposit.amount > 0 {
                        // Exécuter le minage (déjà dans player_mine)
                        mine_deposit(entity, ...);
                        return;
                    }
                }
            }
        }
    }

    // Priorité 4 : Bâtiment avec output (prendre le résultat)
    if keys.just_pressed(KeyCode::KeyE) {
        for (entity, _) in &entities_nearby {
            if let Ok(building) = building_q.get(*entity) {
                if let Ok(mut inv) = commands.get_entity(*entity).map(|e| e.get_mut::<Inventory>()) {
                    // Prendre 1 item du output
                    // ... logique de transfert
                }
            }
        }
    }
}
```

### 3. `T` et `P` supprimés → glisser-déposer déjà en place

**Fichier :** `src/economy/inspect/interaction.rs` (`resource_transfer`)

Supprimer la fonction `resource_transfer` (lignes 276-323) et son enregistrement dans `src/economy/mod.rs`. Le glisser-déposer de l'inventaire le remplace.

### 4. Tooltips au survol

**Nouveau fichier :** `src/rendering/tooltip.rs` (ou ajout à `src/core/tooltip.rs`)

Ajouter un système de tooltip qui s'affiche quand la souris survole un bâtiment :

```rust
pub fn building_tooltip_system(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<MinimapCamera>)>,
    cfg: Res<MapConfig>,
    spatial: Res<SpatialRegistry>,
    building_q: Query<(&Building, Option<&Assembler>, Option<&Inventory>)>,
    registry: Res<BuildingRegistry>,
    mut commands: Commands,
) {
    let Some(TilePosition { x, y }) = cursor_to_tile(&windows, &camera, &cfg) else { return; };
    let Some(entity) = spatial.at(x, y) else { return; };
    let Ok((building, assembler, inv)) = building_q.get(entity) else { return; };

    // Afficher tooltip :
    // - Nom du bâtiment
    // - État (actif/en pause)
    // - Item en cours (si assembleur)
    // - Quantité items si inventaire
    // - Puissance consommée/générée
}
```

**Format du tooltip suggéré :**
```
┌─ Four ─────────────────┐
│ ▶ Actif                │
│ Lingot de fer  (45%)   │
│ ⚡ 8W                 │
└────────────────────────┘
```

### 5. Suppression de `resource_transfer` (T/P)

**Fichier :** `src/economy/inspect/interaction.rs`

Supprimer intégralement la fonction `resource_transfer` et sa déclaration dans `src/economy/mod.rs`.

---

## Résumé des fichiers modifiés

| Fichier | Changement |
|---------|------------|
| `src/economy/inspect/interaction.rs` | Supprimer la vérification de distance pour le clic gauche (lignes 81-89). Supprimer `resource_transfer` (T/P). |
| `src/economy/tiered_structure.rs` | Déplacer la logique d'interaction capsule dans le nouveau système central. |
| `src/economy/player.rs` | Déplacer la logique de minage dans le nouveau système central. Supprimer `player_mine`. |
| `src/player/interact.rs` (NOUVEAU) | Système centralisé `contextual_interact` qui gère la priorité des actions E. |
| `src/rendering/tooltip.rs` (NOUVEAU) | Système de tooltip au survol avec infos bâtiment. |
| `src/economy/mod.rs` | Supprimer l'enregistrement de `player_mine` et `resource_transfer`. Ajouter l'enregistrement de `contextual_interact`. |
| `src/rendering/mod.rs` | Ajouter l'enregistrement du système de tooltip. |

## Impact sur les TOML

**Aucun changement.** Le fichier `map_config.toml` conserve `inspect_range_tiles` pour d'éventuels usages futurs, mais le clic gauche ne l'utilise plus.

Le champ `builder_reach` et `inspect_range_tiles` restent pour le placement de bâtiments et l'interaction `E`.

## Tests de non-régression

1. **Clic gauche sur bâtiment lointain** → le panneau s'ouvre ✅
2. **E près de la capsule** → donne les items, ne mine pas ✅
3. **E près d'un dépôt** (seul) → mine ✅
4. **E près d'un bâtiment** (output) → prend le résultat ✅
5. **E près de la capsule ET d'un dépôt** → donne les items, ne mine pas ✅
6. **Survol d'un bâtiment** → tooltip visible ✅
7. **T et P ne font plus rien** → supprimés ✅
8. **Glisser-déposer inventaire** → fonctionne toujours ✅
