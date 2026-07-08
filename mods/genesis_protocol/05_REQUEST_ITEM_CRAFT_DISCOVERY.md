# Request 05 — Déclencheur de découverte sur craft d'item

## Contexte

Le système de découvertes actuel ne se déclenche que sur l'usage d'un bâtiment (`building` + `threshold`). Pour la Phase 0 du mod Genesis Protocol, on veut que la première découverte se déclenche quand le joueur **craft un item spécifique** (le `hammer`), pas quand il utilise un bâtiment pour la première fois.

## Demande

Ajouter un nouveau type de déclencheur dans le système de découvertes : `type = "craft"` ou `trigger_type = "item"` qui se déclenche quand le joueur craft un item spécifique.

### Format TOML souhaité

```toml
[[discovery]]
building = "workbench"       # optionnel : bâtiment utilisé pour le craft
type = "recipe"              # ce que la découverte débloque (inchangé)
id = "proto_iron"            # recette débloquée (inchangé)
trigger_type = "item"        # NOUVEAU : déclenché par craft d'item
trigger_id = "hammer"        # NOUVEAU : l'item dont le craft déclenche la découverte
threshold = 1                # nombre de crafts nécessaires (défaut: 1)
message = "..."
```

### Comportement attendu

- Quand le joueur craft l'item `trigger_id` (via n'importe quel bâtiment) et que le compteur de crafts atteint `threshold`, la découverte se déclenche
- `building` devient optionnel (si absent, la découverte n'est pas liée à un bâtiment spécifique — juste au craft global)
- Si `trigger_type` est absent, le comportement actuel (basé sur building) est conservé pour la rétrocompatibilité

### Détail d'implémentation possible

Dans `check_discoveries()` dans `src/economy/discovery.rs`, ajouter une vérification alternative : un `CraftCounter` component (similaire à `ProductionCounter`) mis à jour par le système de craft/output. Les découvertes avec `trigger_type = "item"` vérifient ce compteur au lieu du `ProductionCounter` du bâtiment.

Alternative plus simple : étendre `ProductionCounter` pour qu'il soit aussi incrémenté sur le joueur lui-même quand il craft un item. Les découvertes avec `building = "player"` ou sans building se déclencheraient alors sur le craft global.
