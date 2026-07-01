# Coding Conventions — Siege Factory

## Règles générales

### 1. Un système = une responsabilité

```rust
// ❌ Interdit
fn update_all(mut enemies: Query<&mut Enemy>, mut inventories: Query<&mut Inventory>) { ... }

// ✅ Autorisé
fn enemy_pathfinding(...)
fn enemy_damage(...)
fn production_tick(...)
fn inventory_transfer(...)
```

### 2. Communication par Events pour les actions importantes

Tout ce qui est "quelque chose s'est passé" doit être un Event :

```rust
// Events (autorisé)
SpawnWaveEvent, BuildingPlacedEvent, EntityDestroyedEvent, GameOverEvent

// Pas Events (ne pas faire)
UpdatePositionEvent (trop fréquent, utiliser Query directe)
RenderEvent (trop fréquent, Bevy gère ça)
```

### 3. Resources pour l'état partagé, pas de singletons globaux

```rust
// ✅ Autorisé : resources ECS
#[derive(Resource)]
struct GameSeed(u64);

#[derive(Resource)]
struct Selection { entities: Vec<Entity> }

// ❌ Interdit : global state
static GAME_SEED: Mutex<u64> = ...;
```

### 4. Ne pas hardcoder les valeurs de game design

```rust
// ❌ Interdit
fn miner_cost() -> HashMap<ResourceId, u32> {
    HashMap::from([(ResourceId::Ore, 10)])
}

// ✅ Autorisé : lire depuis le registry
fn miner_cost(registry: Res<BuildingRegistry>) -> HashMap<ResourceId, u32> {
    registry.get(BuildingKind::Miner).cost.clone()
}
```

### 5. Séparation logique / rendu

Les systèmes de logique (économie, combat, pathfinding) ne doivent **jamais** toucher aux composants de rendu (`Sprite`, `Transform`, `Handle<Image>`).

```rust
// ❌ Interdit
fn production_system(mut sprites: Query<&mut Sprite>, ...) { ... }

// ✅ Autorisé
fn production_system(mut inventories: Query<&mut Inventory>, ...) { ... }
```

### 6. Systèmes atomiques testables

Chaque système doit pouvoir être testé en isolation :

```rust
// Extraire la logique dans une fonction pure
fn compute_recipe_output(inputs: &Inventory, recipe: &Recipe) -> Option<Inventory> { ... }

// Le système appelle juste la fonction
fn production_system(mut inventories: Query<&mut Inventory>, recipes: Res<RecipeBank>) { ... }

// Le test teste la fonction, pas le système
#[test]
fn test_compute_recipe() { ... }
```

### 7. Nommage

| Élément | Convention | Exemple |
|---|---|---|
| Modules | snake_case | `map/tile_grid.rs` |
| Types | PascalCase | `TileGrid`, `GameState` |
| Fonctions | snake_case | `setup_map`, `handle_tile_click` |
| Events | PascalCase + Event suffix | `SpawnWaveEvent` |
| Components | PascalCase | `TilePosition`, `Inventory` |
| Resources | PascalCase | `BuildingRegistry` |
| SystemSets | PascalCase | `GameStep::Production` |
| Fichiers data | snake_case | `buildings.toml` |

### 8. Tests obligatoires

- Toute fonction pure `compute_*` ou `should_*` : test unitaire obligatoire
- Tout nouveau système ECS : test d'intégration minimum (création App + run + vérification)
- Tout registry : test de chargement depuis TOML
- `proptest` pour les invariants (ex: jamais de ressources négatives)

### 9. Commentaires

- Pas de commentaire "what" (le code explique)
- Commentaire "why" si la raison n'est pas évidente
- `TODO` pour les fonctionnalités futures, avec référence si possible

### 10. Limites

- Fonction : max 30 lignes
- Système ECS : max 15 lignes (hors query definitions)
- Fichier : max 300 lignes
- Si un fichier dépasse, le diviser en plusieurs modules
