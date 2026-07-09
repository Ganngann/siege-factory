# Request 11 — Objectif courant affiché en permanence dans le HUD

## Contexte

Le joueur n'a aucun moyen de savoir quel est son objectif actuel. Les toasts du tutoriel disparaissent après quelques secondes, la capsule n'affiche pas les items requis, et il n'y a pas de rappel visuel.

## Demande

Ajouter un système d'objectifs persistants affichés en permanence dans le HUD (coin supérieur gauche ou droit de l'écran).

### Format TOML

Nouveau fichier `data/objectives.toml` :

```toml
[[objectives]]
id = "welcome"
text = "Réparer la capsule Genesis"
trigger_type = "game_start"

[[objectives]]
id = "craft_hammer"
text = "Fabriquer un marteau à l'Établi pour dégager les débris"
trigger_type = "tutorial_step"
trigger_id = "craft_hammer"

[[objectives]]
id = "deliver_hammer"
text = "Apporter le marteau à la capsule (approche-toi et presse E)"
trigger_type = "item_crafted"
trigger_id = "hammer"

[[objectives]]
id = "tier_rope"
text = "Apporter 3× corde à la capsule pour rétablir l'alimentation primaire"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_0_clear"

[[objectives]]
id = "tier_iron_parts"
text = "Apporter 5× pièces de fer pour restaurer les circuits internes"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_0"

[[objectives]]
id = "tier_gear"
text = "Apporter 3× engrenage + 2× tuyau pour rétablir la pression"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_1"

[[objectives]]
id = "tier_circuit"
text = "Apporter 3× circuit pour remplacer le noyau informatique"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_2"

[[objectives]]
id = "tier_motor"
text = "Apporter 2× moteur + 2× batterie pour débloquer les pompes internes"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_3"

[[objectives]]
id = "tier_processor"
text = "Apporter 2× nano-pack pour souder les microfissures de la coque"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_4"

[[objectives]]
id = "tier_organic"
text = "Apporter 3× composé organique + 1× protéine pour synthétiser les fluides biologiques"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_5"

[[objectives]]
id = "tier_neural"
text = "Apporter 1× interface neurale à la capsule pour amorcer la séquence de réveil"
trigger_type = "tier_unlocked"
trigger_id = "genesis_phase_6"

[[objectives]]
id = "final"
text = "Réparer la capsule. Un premier souffle."
trigger_type = "tier_unlocked"
trigger_id = "genesis_final"
```

### Types de déclencheur

| `trigger_type` | Description |
|---------------|-------------|
| `game_start` | Objectif initial au lancement |
| `tutorial_step` | Se déclenche quand une étape du tutoriel est atteinte (via `trigger_id` = step_id) |
| `item_crafted` | Se déclenche quand le joueur craft l'item `trigger_id` pour la première fois |
| `tier_unlocked` | Se déclenche quand le tier `trigger_id` de la capsule est débloqué |

### Comportement

- Un seul objectif affiché à la fois (le plus récent non complété)
- L'objectif se met à jour automatiquement quand le trigger suivant se déclenche
- Affichage permanent dans le HUD (ne disparaît pas)
- Police lisible, fond semi-transparent, en haut à gauche ou à droite
- L'objectif reste affiché jusqu'à ce qu'il soit remplacé par le suivant
