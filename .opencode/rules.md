# Règles de qualité — Siege Factory

## Principe : zéro solution temporaire
Toute modification proposée doit être définitivement correcte.
Si une approche nécessite un « on améliorera plus tard », un « quick fix » ou un « en attendant »,
c'est qu'elle n'est pas acceptable. Revois ta conception.

## Règles vérifiables

- **Pas de `.unwrap()`** ou `.expect()` dans le code de production (build.rs est exclu)
- **Pas de `#[allow()]`** pour faire taire un warning sans corriger la cause
- **Pas de string en dur** quand la valeur existe déjà dans un registre TOML
- **Pas de `TODO`, `FIXME` ou `HACK`** laissés dans le code après modification
- **Pas de duplication** : si un pattern existe, réutilise-le (SystemParam, type alias, etc.)
- **Toute nouvelle entité data-driven** (building, unité, ressource, recette) doit être testée
- **Tout ajout à un barrel re-export** (`economy/components.rs`) doit être documenté par un `pub use`
