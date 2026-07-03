# Testing Strategy — Siege Factory

## Principes

1. **Toute fonction pure extraite** (`compute_*`, `should_*`) : test unitaire obligatoire.
2. **Tout nouveau système ECS** : test d'intégration minimal (App + run + vérification).
3. **Tout registre data-driven** : test de chargement depuis TOML.
4. **`proptest`** pour les invariants (jamais de ressources négatives, jamais de HP < 0, coordonnées toujours valides).

## Types de tests

### Unitaires (cargo test)

Testent les fonctions isolées, sans ECS.

### Intégration ECS (cargo test — headless)

Testent les systèmes avec `App::new()` sans rendu.

### Propriétés (proptest)

Testent des invariants sur des entrées générées aléatoirement (pas de overflow, pas de ressources négatives).

### Intégration complète

Simulation d'une partie complète avec des données mockées.

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
