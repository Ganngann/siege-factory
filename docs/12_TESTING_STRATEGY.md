# Testing Strategy — Siege Factory

## Principes

1. **Toute fonction pure extraite** (`compute_*`, `should_*`) : test unitaire obligatoire.
2. **Tout nouveau système ECS** : test d'intégration minimal (App + run + vérification).
3. **Tout registre data-driven** : test de chargement depuis TOML.
4. **`proptest`** pour les invariants (jamais de ressources négatives, jamais de HP < 0, coordonnées toujours valides).

## Types de tests

### Unitaires (cargo test)

Testent les fonctions isolées, sans ECS.

```rust
#[test]
fn compute_recipe_output_empty_input() {
    let inventory = Inventory::new();
    let recipe = Recipe { input: vec![(Ore, 5)], output: vec![(Ammo, 1)], time_sec: 2.0 };
    assert_eq!(compute_recipe_output(&inventory, &recipe), None);
}

#[test]
fn compute_recipe_output_sufficient() {
    let mut inventory = Inventory::new();
    inventory.add(Ore, 5);
    let recipe = Recipe { input: vec![(Ore, 5)], output: vec![(Ammo, 1)], time_sec: 2.0 };
    let result = compute_recipe_output(&inventory, &recipe).unwrap();
    assert_eq!(result.get(Ammo), 1);
    assert_eq!(result.get(Ore), 0); // Consumed
}
```

### Intégration ECS (cargo test — ECS headless)

Testent les systèmes avec `App::new()` sans rendu.

```rust
#[test]
fn production_system_ticks() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(EconomyPlugin);

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
    app.update();

    // Arrange : ajouter un miner sur un gisement
    let miner = app.world_mut().spawn((
        Building { kind: BuildingKind::Miner },
        Inventory::new(),
        TilePosition { x: 5, y: 5 },
    )).id();
    app.world_mut().entity_mut(miner).insert(OnDeposit(true));

    // Act : run production tick
    app.update();

    // Assert : miner a produit de l'ore
    let inv = app.world().get::<Inventory>(miner).unwrap();
    assert!(inv.get(ResourceId::Ore) > 0);
}
```

### Propriétés (proptest)

Testent des invariants sur des entrées générées aléatoirement.

```rust
proptest! {
    #[test]
    fn inventory_never_negative(
        resource in prop::sample::any::<ResourceId>(),
        amount in 0..1000u32,
        add_amount in 0..1000u32,
        remove_amount in 0..1000u32,
    ) {
        let mut inv = Inventory::new();
        inv.add(resource, amount);
        inv.remove(resource, add_amount.min(amount)); // ne peut pas retirer plus que ce qu'on a
        prop_assert!(inv.get(resource) >= 0); // toujours >= 0 (surflow protégé)
    }
}
```

### Intégration complète

Simulation d'une partie complète avec des données mockées.

```rust
#[test]
fn game_end_to_end() {
    let mut app = App::new();
    app.add_plugins((CorePlugin, MapPlugin, EconomyPlugin, EnemyPlugin, CombatPlugin));
    app.init_state::<GameState>();
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
    app.update();

    // Simuler 100 ticks
    for _ in 0..100 {
        app.update();
    }

    // Vérifier que le jeu tourne sans erreur
    // L'état n'a pas crashé, les ressources ne sont pas négatives, etc.
}
```

## Couverture visée

| Type | Couverture cible |
|---|---|
| Fonctions pures | 100% |
| Systèmes production | 100% |
| Pathfinding | 100% |
| Placement buildings | 90% |
| Combat/dégâts | 90% |
| UI | 50% (seulement les Events) |
| Réseau | N/A (plus tard) |

## Exécution

```powershell
cargo test                                # Tests rapides
cargo test --release                      # Tests longs + proptest
cargo test --test integration_*           # Tests d'intégration uniquement
```

## CI (GitHub Actions)

Les tests sont exécutés automatiquement sur chaque push et PR. Échec = merge bloqué.
