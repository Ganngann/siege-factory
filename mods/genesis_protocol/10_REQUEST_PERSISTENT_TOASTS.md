# Request 10 — Toasts persistants (tutoriel / narratif)

## Contexte

Les toasts du tutoriel (`tutorial.toml`) et les toasts narratifs disparaissent automatiquement après `lifetime` secondes. Le joueur n'a pas le temps de lire les textes longs (surtout les textes narratifs de 30+ mots).

## Demande

Ajouter un champ optionnel `persistent: bool` aux étapes de tutoriel :

```toml
[[steps]]
id = "welcome"
toast = "Année 2147..."
persistent = true    # ← reste affiché jusqu'à dismiss
condition = "player_moved_distance"
params = { distance = "1" }
```

### Comportement attendu

- Si `persistent = true` : le toast reste à l'écran indéfiniment
- Le joueur le ferme avec un clic gauche ou la touche Espace/Entrée
- Tant qu'il n'est pas fermé, les autres toasts du tutoriel ne s'affichent pas (file d'attente bloquante)
- Si `persistent` est absent ou `false` : comportement actuel (auto-disparition après `lifetime`)

### Alternatives possibles

**Option 1 — `persistent` par toast** (recommandé) :
Ajouter le champ dans `TutorialStep` et dans le système d'affichage des toasts.

**Option 2 — `lifetime = 0`** :
Interpréter `lifetime = 0` comme « ne disparaît jamais ». Plus simple côté code, moins explicite dans les TOML.

**Option 3 — Dismiss global** :
Tous les toasts du tutoriel sont persistants jusqu'à clic. Plus simple mais moins flexible.

### Détail d'implémentation

Dans `src/core/toast.rs` (ou équivalent) :
- Ajouter un champ `persistent: bool` à la structure `Toast`
- Modifier le système d'affichage : si `persistent`, ne pas décrémenter le timer
- Ajouter un système d'input qui détecte le clic/barre d'espace sur les toasts persistants → les retire
- Bloquer l'affichage des nouveaux toasts si un toast persistant est actif
