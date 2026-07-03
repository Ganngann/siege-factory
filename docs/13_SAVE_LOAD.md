# Save & Load — Siege Factory

## Statut

Pas encore implémenté. Le save/load est prévu après la stabilisation du socle solo.

## Format (planifié)

Sérialisation binaire via `bincode` ou `postcard`. L'état complet du monde ECS est sérialisé :

- Resources : `GameState`, `GameSeed`
- Ressources économiques : `Inventory` de chaque entité
- Buildings : position, type, HP, inventaire interne
- Ennemis : type, position, HP, path actuel
- Vagues : état actuel, timer

Pas de sérialisation des entités éphémères (projectiles, particules).

## Emplacement (planifié)

- Windows : `%APPDATA%/siege-factory/saves/`
- Portable via `dirs::data_dir()`

## Évolution

- **Phase 1** (scaffold TD) : pas de save/load nécessaire (replay rapide)
- **Phase 2** (Factorio) : save incrémental par chunk pour carte infinie
