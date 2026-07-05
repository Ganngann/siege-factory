# Scaling Belt — Siege Factory

## Objectif

Milliards d'items sur la carte. Usines automatiques complexes. 60+ FPS.
Un item seul sur une boucle de belt à l'autre bout de la map, avec un détecteur
qui active une lampe de ton côté : la lampe clignote en temps réel, le compteur
du détecteur est exact après 1 heure.

## Pourquoi le système actuel ne scale pas

### Stockage mémoire

```
1 item = 1 struct ItemOnBelt { resource_id: String, acc: f32 } ≈ 32 bytes
1 milliard d'items × 32 = 32 GB  →  impossible
```

Stockage par tuile (`Vec<Option<ItemOnBelt>>` par belt) = O(nombre d'items).
Au-delà de ~10M items, la RAM explose.

### CPU

```
1 milliard d'items × 20 Hz × ~5 ops = 100 milliards ops/sec  →  impossible
```

Itérer chaque item individuellement à chaque tick ne tient pas à l'échelle.

### Rendu

1 sprite par item = 1 draw call par item. Au-delà de ~10 000 sprites visibles,
le GPU sature. C'est le bottleneck numéro 1 aujourd'hui.

## La solution : ItemBlock + BeltSegment

### Concept central

On arrête de stocker les items individuellement. On stocke des **blocs
d'items** — des groupes contigus d'items identiques (ou de patterns répétés)
sur un **segment** de belt.

Un **segment** = suite continue de belts de même direction et même vitesse,
sans splitter/sorter/building au milieu.

```
BeltSegment (direction Est, speed 2.0, 10 tuiles × 5 slots = 50 slots)
  └── ItemBlock { pattern: [fer], repetition: 50, front_slot: 0 }
       → 50 items de fer, stockés en ~40 bytes au lieu de 50 × 32 = 1600 bytes
```

### Mémoire : de O(items) à O(blocs)

| Scénario | Items individuels | Blocs |
|----------|-------------------|-------|
| 50 fer sur 1 belt | 1.6 KB | 40 B |
| 1M fer sur 1000 belts saturés | 32 MB | 40 KB |
| 1 milliard fer (200k belts saturés) | 32 GB | ~8 MB |

Le pire cas (items tous différents, zéro répétition) dégénère en O(1) bloc par
item. Mais les usines réelles produisent des longues files homogènes →
compression massive par construction.

### CPU : de O(items/tick) à O(blocs/tick)

Chaque tick à 20 Hz avance le `front_slot` de chaque bloc. O(nombre de blocs),
pas O(nombre d'items). 200k blocs × 20 Hz = 4M ops/sec — trivial.

## Structures de données

### ItemBlock

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemBlock {
    /// Pattern de ressources répété. Pour un bloc homogène : `vec![ResourceId("fer")]`.
    /// Pour un pattern périodique : `vec![fer, fer, cuivre]` (période 3).
    /// Longueur = `pattern.len()`.
    pub pattern: Vec<ResourceId>,

    /// Nombre de fois que le pattern se répète.
    /// Nombre total d'items = `pattern.len() * repetition`.
    pub repetition: u32,

    /// Position du premier item du bloc le long du segment, en slots (0 = entrée).
    /// Avance de `speed * slot_count_per_tile * dt` à chaque tick.
    pub front_slot: f32,
}
```

**Invariant** : `front_slot >= 0.0` et
`front_slot + (pattern.len() as f32 * repetition as f32) <= segment.slot_count as f32 + tolerance`.
Un bloc ne dépasse jamais la fin du segment — quand le front sort, les items
sont transférés au segment suivant.

**Calcul de la position d'un item individuel dans le bloc** :
```
item_i (0-indexed) est à la position : front_slot + i
sa resource est : pattern[i % pattern.len()]
```

### BeltSegment

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeltSegment {
    /// ID unique du segment.
    pub id: u32,

    /// Direction de toutes les tuiles du segment.
    pub direction: Direction,

    /// Vitesse de la belt (slots/seconde).
    pub speed: f32,

    /// Nombre total de slots dans le segment = nb_tuiles × slots_par_tuile.
    pub slot_count: u32,

    /// Liste des tuiles appartenant à ce segment (pour le rendu et l'inspect).
    pub tiles: Vec<(i32, i32)>,

    /// Blocs d'items sur ce segment, triés par front_slot décroissant
    /// (le bloc le plus en avant est en premier).
    pub blocks: Vec<ItemBlock>,

    /// Segments de sortie (ceux connectés à la sortie de ce segment).
    /// 1 pour un belt normal, 2+ pour un splitter.
    pub output_segments: Vec<u32>,

    /// Segments qui feedent ce segment en entrée (pour le réveil time-skip).
    pub input_segments: Vec<u32>,

    /// Timestamp du dernier tick simulé (en secondes depuis le démarrage).
    /// Utilisé pour le time-skip : si `now - last_tick > 5.0` et
    /// `observers.is_empty()`, alors le segment est dormant et peut être
    /// sauté analytiquement au prochain réveil.
    pub last_tick: f32,

    /// Positions (en slots le long du segment) des détecteurs attachés.
    /// VIDE pour l'instant — interface pour le futur réseau électrique.
    /// Un segment avec observers non-vide ne JAMAIS être time-skippé.
    pub observers: Vec<u32>,
}
```

**Invariants critiques** :
1. Tous les blocs sont dans l'ordre `front_slot` décroissant
2. Deux blocs ne se chevauchent jamais (gap ≥ 0 entre la fin d'un bloc et le
   début du suivant)
3. `blocks[i].front_slot + blocks[i].len() <= slot_count` (transfert géré avant
   débordement)

### BeltGrid (ressource)

```rust
#[derive(Resource)]
pub struct BeltGrid {
    /// Tous les segments, indexés par ID.
    pub segments: HashMap<u32, BeltSegment>,

    /// Map : position de tuile (tx, ty) → ID du segment propriétaire.
    /// Permet de retrouver le segment d'une tuile en O(1).
    pub tile_to_segment: HashMap<(i32, i32), u32>,

    /// Compteur d'IDs pour allouer de nouveaux segments.
    next_id: u32,
}
```

### SlotConfig (config par tuile, pas par segment)

La configuration des slots (positions visuelles, nombre de slots par tuile)
rester sur le component `BeltSlots` de chaque entité belt. Le segment stocke
`slot_count` (total du segment) mais pas les positions individuelles — celles-ci
sont dérivées de la tuile via `compute_slot_positions` existant.

```rust
// Inchangé — reste sur chaque entité belt.
#[derive(Component)]
pub struct BeltSlots {
    pub direction: Direction,
    pub slot_positions: Vec<Vec2>,  // positions visuelles des slots de CETTE tuile
    pub speed: f32,
}
```

## Construction des segments

### Détection automatique

À chaque pose ou déconstruction de belt, on recalcule les segments affectés.

**Règles de regroupement** : deux tuiles belt appartiennent au même segment SI :
1. Même direction
2. Même vitesse
3. Adjacentes (la sortie de l'une = l'entrée de l'autre)
4. Pas de splitter, sorter, ou building entre les deux

Un splitter/sorter/building est un **endpoint** de segment. La tuile qui
contient un splitter est elle-même un segment de longueur 1 (slot_count =
slots_par_tuile).

### Algorithme de construction

```
fn rebuild_segments_around(belt_grid, tx, ty):
    1. Trouver le point de départ : remonter dans la direction opposée
       tant que les tuiles adjacentes sont belt + même direction + même vitesse
       et ne sont pas un splitter/sorter.
    2. Depuis le point de départ, descendre dans la direction jusqu'à :
       - un changement de direction
       - un changement de vitesse
       - un splitter/sorter/building
       - la fin d'un belt
    3. Toutes les tuiles parcourues = un nouveau segment.
    4. Supprimer les anciens segments qui chevauchaient ces tuiles.
    5. Calculer output_segments et input_segments en regardant les voisins.
```

### Fusion / scission

- **Pose d'un belt** : peut fusionner 1 ou 2 segments existants (si le nouveau
  belt connecte deux segments de même direction/vitesse).
- **Déconstruction** : scinde un segment en 0, 1 ou 2 sous-segments. Les items
  sont redistribués dans les sous-segments selon leur position.

## Simulation

### Tick-by-tick (FixedUpdate, 20 Hz)

```rust
fn advance_belt_segments(
    time: Res<Time<Fixed>>,
    mut belt_grid: ResMut<BeltGrid>,
    // ... queries pour buildings, splitters, sorters ...
) {
    let dt = time.delta_secs();
    let now = current_time();

    for (_, segment) in &mut belt_grid.segments {
        // 1. Si dormant et skippable, skip
        if now - segment.last_tick > 5.0
           && segment.observers.is_empty()
           && segment.input_segments.is_empty() {
            // Time-skip : calculer analytiquement
            let elapsed = now - segment.last_tick;
            for block in &mut segment.blocks {
                block.front_slot += segment.speed * elapsed;
            }
            // Gérer les débordements (voir section time-skip)
            segment.last_tick = now;
            continue;
        }

        // 2. Simulation tick-par-tick normale
        let step = segment.speed * dt;
        for block in &mut segment.blocks {
            block.front_slot += step;
        }
        segment.last_tick = now;

        // 3. Notifier les observers (détecteurs futurs)
        // Pour chaque observer à position D :
        //   si un bloc a crossed D ce tick, émettre un événement.

        // 4. Transférer les items qui sortent du segment
        transfer_excess_items(segment, &mut belt_grid, ...);
    }
}
```

### Transfert de blocs entre segments

Quand le front d'un bloc dépasse `slot_count` :
```
slots_débordés = front_slot + block.len() - slot_count
items_transférés = min(slots_débordés, block.len())
```

Les items transférés vont vers `output_segments[0]` (ou répartis pour
splitter). Le bloc source est réduit : soit `repetition` diminue,
soit le bloc est entièrement consommé et retiré.

**Cas des blocs avec pattern** : si on transfère 7 items d'un bloc
`pattern=[a,a,b], repetition=10` (30 items), il reste 23 items = 7
périodes complètes + 2 items résiduels `[a,a]`. Le bloc devient :
```
pattern = [a,a], repetition = 23/2 → non, 23 = 7×3 + 2
→ pattern = [a,a], repetition = 7, puis bloc résiduel [a,a] repetition=1
```
Plus simple : on décompose en `floor(remaining / pattern_len)` répétitions
complètes + 1 bloc résiduel si besoin.

### Splitter au niveau bloc

Un splitter a 2+ output_segments. La répartition se fait au niveau bloc :
```
bloc_entrant → diviser en 2 blocs vers les 2 outputs
  output 0 : pattern copié, repetition = ceil(repetition/2)
  output 1 : pattern copié, repetition = floor(repetition/2)
  counter interne pour alterner
```

Le round-robin se fait au niveau **bloc**, pas item-par-item. C'est une
approximation acceptable car les blocs homogènes donnent le même résultat
que le round-robin item-par-item.

### Sorter au niveau bloc

Un sorter route selon le resource_id. Au niveau bloc :
- Si tout le pattern match le filtre → bloc entier vers le side output
- Si rien ne match → bloc entier vers le forward output
- Si mixte → décomposer le bloc en sous-blocs par resource, router chacun

### Building output → segment

Quand un building (miner, assembler) produit un item :
1. Trouver le segment connecté à la sortie du building
2. Ajouter un `ItemBlock { pattern: [res], repetition: 1, front_slot: 0 }`
   au début du segment (ou fusionner avec un bloc existant à la position 0
   si même resource)

### Segment → building input

Quand un bloc atteint la fin d'un segment qui sort sur un building :
1. Consommer les items (ajouter à l'inventaire du building)
2. Réduire le bloc (diminuer repetition ou retirer)

### Collisions entre blocs

Un bloc ne peut pas avancer dans un bloc devant lui. Si
`block[i].front_slot + step > block[i-1].back_slot`, le bloc s'arrête juste
derrière le bloc précédent. Les blocs sont triés par front_slot décroissant,
donc on itère du premier (le plus en avant) au dernier.

```
for i in 0..blocks.len():
    target = blocks[i].front_slot + step
    if i > 0:
        max_pos = blocks[i-1].front_slot - blocks[i-1].len()
        target = target.min(max_pos)
    blocks[i].front_slot = target
```

## Time-skip (optimisation pour segments dormants)

### Conditions

Un segment est **dormant** si :
1. `now - last_tick > 5.0` (au moins 5 sec sans simulation)
2. `observers.is_empty()` (aucun détecteur attaché)
3. `input_segments` ne contient que des segments eux-mêmes dormants

### Calcul

Au prochain réveil (input reçu, ou segment chargé dans le viewport) :
```
elapsed = now - last_tick
for block in &mut blocks:
    block.front_slot += speed * elapsed
// Gérer les débordements → compteurs de passage
// Gérer les loops → incrementer le compteur de tours
```

**O(1) par bloc**, quel que soit `elapsed`. Pour une loop de 1 item pendant
1 heure : 1 calcul, pas 72000 itérations.

### Loops et compteurs

Un segment dont l'output revient à son input (composante fortement connexe de
taille 1) est une **loop**. Les items qui sortent ré-entrent immédiatement.

Pour le time-skip d'une loop avec 1 item :
```
loop_length = slot_count
speed = segment.speed
elapsed = now - last_tick
passages_complets = floor((front_slot + speed * elapsed) / loop_length)
                     - floor(front_slot / loop_length)
front_slot = (front_slot + speed * elapsed) % loop_length
// Les détecteurs futurs sur cette loop recevraient
// `passages_complets` événements.
```

**C'est exact.** Pas d'approximation, pas de perte.

### Pourquoi les observers désactivent le time-skip

Un détecteur doit émettre un événement **au moment précis** où l'item passe.
En time-skip, on sait combien de fois l'item est passé, mais pas à quel
timestamp exact. Pour pouvoir rejouer les événements en temps réel (lampe qui
clignote), il faut simuler tick-par-tick.

Les segments avec observers simulent donc à 20 Hz en permanence. Comme les
détecteurs sont posés par le joueur (au pire quelques milliers), le coût reste
négligeable.

## Détection de patterns (compression des blocs)

### Quand déclencher

Après qu'un bloc a reçu de nouveaux items (fusion avec un bloc entrant), on
vérifie si la séquence résultante contient une période détectable.

### Algorithme

Utiliser la **Z-function** (ou KMP failure function) sur la séquence de
resources observée :

```
fn detect_pattern(seq: &[ResourceId]) -> Option<(Vec<ResourceId>, u32)>:
    1. Calculer la Z-function.
    2. Trouver la plus petite période p telle que seq[p..] == seq[..n-p].
    3. Si p < n/2 (au moins 2 répétitions) :
       retourner (seq[..p].to_vec(), (n/p) as u32)
    4. Sinon : pas de pattern détectable, stocker tel quel.
```

### Exemples

```
[fer, fer, fer, fer, fer, fer]
  → pattern = [fer], repetition = 6 ✅ (forte compression)

[fer, fer, cuivre, fer, fer, cuivre, fer, fer, cuivre]
  → pattern = [fer, fer, cuivre], repetition = 3 ✅

[fer, cuivre, fer, fer, cuivre]
  → pattern = [fer, cuivre], repetition = 2 + résiduel [fer]
  → 2 blocs : [fer, cuivre]×2 et [fer]×1
```

### Quand NE PAS détecter

- Séquence de moins de 4 items : pas assez de données pour confirmer une période
- Pattern trop court (len 1 avec repetition 2) : pas intéressant
- Pattern qui contient lui-même une sous-période : décomposer d'abord

## Rendu (viewport-only)

### Principe

Les sprites ne sont créés que pour les segments dont au moins une tuile est
dans le viewport + marge. Les items sont rendus par their position dans le bloc.

### sync_segment_sprites (Update, 60 Hz)

```rust
fn sync_segment_sprites(
    mut commands: Commands,
    belt_grid: Res<BeltGrid>,
    cfg: Res<MapConfig>,
    camera: Query<(&Camera, &GlobalTransform)>,
    belt_query: Query<(&TilePosition, &BeltSlots)>,
    mut segment_sprites: ResMut<SegmentSpriteCache>,
) {
    let viewport = compute_visible_tiles(camera, cfg);

    for (id, segment) in &belt_grid.segments {
        let in_view = segment.tiles.iter()
            .any(|(tx, ty)| viewport.contains(*tx, *ty));

        if in_view {
            // Créer/mettre à jour les sprites pour chaque item visible
            render_segment_items(segment, belt_query, &mut segment_sprites, ...);
        } else {
            // Détruire les sprites de ce segment
            segment_sprites.despawn_segment(id, &mut commands);
        }
    }
}
```

### render_segment_items

Pour chaque bloc, calculer la position visuelle de chaque item :
```
for block in &segment.blocks:
    for i in 0..block.len():
        slot_pos = block.front_slot + i as f32
        tile_index = floor(slot_pos / slots_per_tile)
        local_slot = slot_pos % slots_per_tile

        // Position visuelle = interpolation entre slot actuel et suivant
        tile = segment.tiles[tile_index]
        slot_positions = belt_query.get(tile).slot_positions
        exact_pos = lerp(slot_positions[local_slot], ...)

        // Resource = pattern[i % pattern.len()]
        resource = block.pattern[i % pattern.len()]
        spawn_or_update_sprite(exact_pos, resource)
```

**Intégration avec budget de draw calls** : si le nombre de sprites à rendre
dépasse un seuil (ex: 5000 visibles), regrouper les items proches en un seul
sprite de "flux" (pastille de couleur). Au-delà de 50 000, ne plus rien rendre
(la belt elle-même reste visible, juste les items disparaissent).

### animate_segment_sprites (Update, 60 Hz)

Interpolation entre la position du tick N-1 et N pour un mouvement fluide à
60 FPS alors que la simulation est à 20 Hz.

```
visible_pos = lerp(pos_tick_N-1, pos_tick_N, alpha)
où alpha = (now - last_fixed_time) / fixed_dt
```

### SegmentSpriteCache

Les sprites sont cachés par segment pour éviter de les recréer à chaque frame.
Quand un segment quitte le viewport, ses sprites sont despawnés.

```rust
#[derive(Resource, Default)]
pub struct SegmentSpriteCache {
    pub sprites: HashMap<u32, Vec<Entity>>,  // segment_id → sprite entities
}
```

## LOD (scoping du viewport)

### belt_chunk_lod (Update, 60 Hz)

Ce système calcule le viewport et gère le cycle dormant/réveil des segments.

```rust
fn belt_chunk_lod(
    mut belt_grid: ResMut<BeltGrid>,
    camera: Query<(&Camera, &GlobalTransform)>,
    cfg: Res<MapConfig>,
    windows: Query<&Window>,
) {
    let viewport = compute_visible_tiles(camera, cfg, windows);

    // Pour chaque segment hors viewport :
    //   - Marquer comme "peut time-skip" si pas d'observers
    // Pour chaque segment dans viewport :
    //   - S'il était dormant, le réveiller (déjà géré par last_tick)
    // Le rendu est géré par sync_segment_sprites séparément.
}
```

Le LOD ne touche **jamais** aux données des items. Il ne fait que :
1. Permettre au rendu de savoir quels segments afficher
2. Permettre au time-skip de savoir quels segments peuvent dormir

## Placement et déconstruction

### Pose d'un belt

1. Spawn l'entité belt (Building, BeltSlots, TilePosition, etc.) — inchangé
2. Appeler `belt_grid.rebuild_segments_around(tx, ty)`
3. Cette fonction :
   a. Trouve les segments voisins (devant/derrière)
   b. Si fusion possible (même direction, même vitesse, pas de splitter) :
      - Fusionner les segments
      - Redistribuer les blocs selon les nouvelles positions
   c. Sinon : créer un nouveau segment pour cette tuile

### Déconstruction d'un belt

1. Appeler `belt_grid.remove_tile_from_segment(tx, ty)`
2. Cette fonction :
   a. Trouve le segment propriétaire
   b. Scinde le segment en 0, 1 ou 2 sous-segments
   c. Redistribuer les blocs selon leur position :
      - Blocs entièrement avant la tuile supprimée → sous-segment avant
      - Blocs entièrement après → sous-segment après
      - Blobs à cheval → items divisible aux slots de la tuile supprimée,
        détruire la fraction sur la tuile supprimée
3. Despawn l'entité belt - inchangé

### Pose/déconstruction d'un splitter/sorter/building

Ces bâtiments sont des endpoints de segment. Leur pose/déconstruction
force un rebuild des segments connectés.

Référer vers le point d'entrée unique `rebuild_segments_around(tx, ty)`.

## Save / Load

### Sérialisation

```rust
#[derive(Serialize, Deserialize)]
pub struct BeltGridSave {
    pub segments: Vec<SegmentSave>,
    pub tile_map: Vec<((i32, i32), u32)>,  // tile → segment_id
}

#[derive(Serialize, Deserialize)]
pub struct SegmentSave {
    pub id: u32,
    pub direction: Direction,
    pub speed: f32,
    pub slot_count: u32,
    pub tiles: Vec<(i32, i32)>,
    pub blocks: Vec<ItemBlock>,
    pub output_segments: Vec<u32>,
    pub input_segments: Vec<u32>,
    pub last_tick: f32,
    pub observers: Vec<u32>,
}
```

### Chargement

1. Reconstruire `BeltGrid` depuis `BeltGridSave`
2. Ne pas reconstruire les segments (ils sont déjà sérialisés)
3. Les entités belt sont respawnées séparément (BuildingSave existant)
   avec leur `BeltSlots` component (direction, slot_positions, speed)

Le `tile_to_segment` est reconstruit depuis `tile_map`.

## Inspect

### Affichage par segment

Quand on inspecte un belt, on retrouve son segment via
`tile_to_segment[(tx, ty)]` et on affiche :

```
Segment #42  | Direction: East  | Speed: 2.0  | 12 tuiles, 60 slots
  Bloc 1:  fer × 50     front @ slot 45
  Bloc 2:  cuivre × 3   front @ slot 10
  Bloc 3:  [pattern fer,fer,cuivre] × 4   front @ slot 0
  Total items: 50 + 3 + 12 = 65 / 60 slots  →  items en attente de transfert
```

Le compteur `Items in transit` devient `blocks.iter().map(|b| b.len()).sum()`.

## Observers (interface pour détecteurs futurs)

### Notes

Ce document fournit l'interface, pas l'implémentation du réseau électrique.

```rust
impl BeltGrid {
    /// Attacher un détecteur à une position d'un segment.
    /// Le segment ne pourra plus être time-skippé.
    pub fn add_observer(&mut self, segment_id: u32, slot_position: u32) {
        if let Some(seg) = self.segments.get_mut(&segment_id) {
            seg.observers.push(slot_position);
        }
    }

    /// Détacher un détecteur.
    pub fn remove_observer(&mut self, segment_id: u32, slot_position: u32) {
        if let Some(seg) = self.segments.get_mut(&segment_id) {
            seg.observers.retain(|&p| p != slot_position);
        }
    }
}
```

Les événements de passage seront émis par `advance_belt_segments` quand un
bloc crossed une position d'observer. Le réseau électrique futur consommera ces
événements via un channel Bevy (EventReader/EventWriter) ou une callback.

## Récapitulatif des systèmes

| Système | Schedule | Rôle |
|---------|----------|------|
| `rebuild_segments_around` | sur pose/déconso | Maintient le graphe de segments |
| `advance_belt_segments` | FixedUpdate 20 Hz | Avance les blocs, gère transferts, time-skip |
| `building_output_to_segment` | FixedUpdate 20 Hz | Émet les items des buildings vers les segments |
| `segment_to_building_input` | FixedUpdate 20 Hz | Consomme les items en sortie de segment vers buildings |
| `belt_chunk_lod` | Update 60 Hz | Gère dormant/réveil, viewport scope |
| `sync_segment_sprites` | Update 60 Hz | Crée/detruit les sprites selon viewport |
| `animate_segment_sprites` | Update 60 Hz | Interpolation 60 FPS des sprites |

## Migration depuis le code actuel

### À supprimer
- `BeltChunk` et `BeltChunkState` (chunk-based storage) dans `belt_grid.rs`
- `BeltCell` (stockage par tuile)
- `compress_chunk` / `reconstruct_chunk` (le Flux chunk n'existe plus)
- `belt_chunk_lod` actuel (remplacé par le nouveau)

### À refactorer
- `advance_belt_slots` → `advance_belt_segments` (itération sur segments, pas tuiles)
- `building_output_tick` → `building_output_to_segment`
- `sync_belt_slot_sprites` → `sync_segment_sprites`
- `animate_belt_positions` → `animate_segment_sprites`
- `placement.rs` : appeler `rebuild_segments_around` après pose/déconso
- `save_load.rs` : sérialiser `BeltGridSave` au lieu de `BeltSave` par tuile
- `inspect.rs` : afficher les blocs du segment au lieu des slots individuels

### À créer
- `belt_segment.rs` : structures `ItemBlock`, `BeltSegment`, `BeltGrid` rewrite
- `belt_pattern.rs` (optionnel) : détection de patterns (Z-function)

### Inchangé
- `BeltSlots` component sur les entités (direction, slot_positions, speed)
- `Building`, `Splitter`, `Sorter` components
- `SpatialRegistry` (lookup position → entity)
- `MapConfig`, `ChunkGrid` (carte)

## Pourquoi cette architecture est scale-proof

1. **Mémoire O(blocs)** : 1 milliard de fer sur 200k belts saturés = ~8 MB
2. **CPU O(blocs/tick)** : 200k blocs × 20 Hz = 4M ops/sec (trivial)
3. **Rendu O(viewport)** : sprites seulement pour les segments visibles
4. **Time-skip O(1)** : segments dormant calculés analytiquement au réveil
5. **Précision exacte** : pas d'approximation, pas de freeze, pas de perte
6. **Patterns périodiques** : `aabaabaab` stocké en 1 bloc, pas 9
7. **Observers** : détecteurs futurs garantis temps réel sans casser le time-skip

### Limites assumées

- **Items tous différents (random)** : dégénère en O(1 bloc par item). Rare en pratique.
- **Splitter round-robin au niveau bloc** : pas item-par-item. Approximation acceptable pour blocs homogènes.
- **Time-skip désactivé si observers** : segments avec détecteurs simulent à 20 Hz en continu. Coût négligeable (peu de détecteurs).

## Notes pour l'implémentation

- Commencer par `ItemBlock` et `BeltSegment`, avec tests unitaires sur
  `front_slot` advancement et block collision.
- Implémenter `rebuild_segments_around` avec tests sur fusion/scission.
- Implémenter `advance_belt_segments` avec tests sur transfert entre segments.
- Implémenter le rendu viewport-only.
- Implémenter le time-skip en dernier (optimisation, pas de correction).
- Les tests unitaires : `cargo test` doit passer avec les nouveaux systèmes.
- Convention : 1 système = 1 responsabilité. Pas de logique dans le rendu.