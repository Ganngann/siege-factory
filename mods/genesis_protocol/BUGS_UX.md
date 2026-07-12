# Bugs & UX — Genesis Protocol

---

## 1. [Feature] Panneau latéral modulaire

**Constat** : Les toasts disparaissent sans laisser de trace, le message "Année 2147" reste bloqué, l'objectif HUD est une ligne de texte sans structure. Trois systèmes différents pour de la communication au joueur, aucun n'est satisfaisant.

**Principe** : Un panneau latéral gauche modulaire, data-driven, composé de modules indépendants définis dans `panel_hud.toml`.

### Structure TOML

Le panneau est un frame contenant des modules empilés verticalement :

```toml
[[sections]]
type = "side_panel"
title = "JOURNAL DE BORD"
width = 280
modules = [
    { type = "objective", title = "OBJECTIF PRINCIPAL" },
    { type = "toast_log", title = "MESSAGES", max_entries = 10 },
    { type = "tutorial_step", title = "INDICE" },
]
```

Chaque module est un composant indépendant, développé séparément :

| Module | Rôle | Données |
|--------|------|---------|
| `objective` | Affiche l'objectif principal courant | `objective.current` (via UiDataContext) |
| `toast_log` | Historique des toasts récents, ordonnés, scrollable | Registre partagé `GlobalMessageLog` |
| `tutorial_step` | Affiche l'étape active du tutoriel | `TutorialState` |

### Modules détaillés

#### Module `objective`

```toml
{ type = "objective", title = "OBJECTIF PRINCIPAL" }
```

- Affiche le texte de l'objectif courant (depuis `ObjectiveState.active_text`)
- Surveille les changements via `Changed<ObjectiveState>`

#### Module `toast_log`

```toml
{ type = "toast_log", title = "MESSAGES", max_entries = 10 }
```

- Affiche les N derniers messages dans l'ordre chronologique
- Les messages viennent d'un registre partagé `GlobalMessageLog` (Resource)
- Le système toast continue de fonctionner mais pousse aussi dans le registre
- `persistent = true` dans le tutoriel n'est plus nécessaire : tous les messages atterrissent dans le log
- Supprimer `persistent = true` de `tutorial.toml` (la persistence est gérée par le registre)

#### Module `tutorial_step` (optionnel)

```toml
{ type = "tutorial_step", title = "INDICE" }
```

- Affiche l'étape active du tutoriel en cours
- Peut être masqué une fois le tutoriel terminé

### Registre partagé : `GlobalMessageLog`

Nouvelle Resource dans `src/core/toast.rs` (ou nouveau fichier) :

```rust
#[derive(Resource, Default)]
pub struct GlobalMessageLog {
    pub messages: Vec<LogEntry>,
}

pub struct LogEntry {
    pub timestamp: f64,   // temps de jeu
    pub text: String,
    pub category: String, // "tutorial", "objective", "system", "event"
}
```

Alimenté par :
- `ToastQueue` → chaque toast est aussi ajouté au log
- `advance_objectives` → les changements d'objectifs sont loggés
- `tutorial_tick` → les étapes de tutoriel sont loggées

### Dépendances d'implémentation

- Nouveau composant TOML `"side_panel"` : container avec modules enfants
- Nouveaux sous-composants : `"objective"`, `"toast_log"`, `"tutorial_step"`
- Nouveau registre `GlobalMessageLog`
- Modifier `toast.rs` pour alimenter le registre
- Modifier `objective.rs` pour alimenter le registre
- Modifier `tutorial.rs` pour alimenter le registre
- Supprimer `spawn_objective_hud` / `update_objective_hud` / `despawn_objective_hud`
- `panel_hud.toml` : remplacer l'actuel `hud_text` par la section `side_panel`

### Fichiers

| Fichier | Action |
|---------|--------|
| `src/core/message_log.rs` | Créer : `GlobalMessageLog` Resource + `LogEntry` |
| `src/ui/components/side_panel.rs` | Créer : composant conteneur `"side_panel"` |
| `src/ui/components/mod.rs` | Ajouter `pub mod side_panel;` |
| `src/ui/mod.rs` | Enregistrer `SidePanelComponent` |
| `src/ui/components/toast.rs` | Modifier : pousser dans `GlobalMessageLog` |
| `src/player/objective.rs` | Modifier : pousser dans `GlobalMessageLog` |
| `src/core/tutorial.rs` | Modifier : pousser dans `GlobalMessageLog` |
| `src/player/objective.rs` | Supprimer `spawn/update/despawn_objective_hud` |
| `panel_hud.toml` | Ajouter la section `side_panel` avec ses modules |
| `tutorial.toml` | Supprimer `persistent = true` (géré par le log) |

---

## 2. [UX] Build bar collée au bas de l'écran

**Problème** : `build_bar.rs:111,388` → `bottom: Val::Px(0.0)`.

**Solution** : Ajouter `bottom_margin` configurable dans `panel_build_bar.toml` et lu par `BuildBarConfig`.

```toml
[[sections]]
type = "build_bar"
bottom_margin = 4
```

**Fichier** : `src/ui/components/build_bar.rs`

---

## 3. [UX] Breadcrumb décale tout le menu

**Problème** : `build_bar.rs:149-160` — le breadcrumb est dans le flux Column et pousse les boutons.

**Solution** : Position absolute du breadcrumb par-dessus la barre.

**Fichier** : `src/ui/components/build_bar.rs`

---

## 4. [Bug] Fenêtre d'inspection centrée

**Problème** : `ui/engine.rs:31-32` → résolution hardcodée 1280×720.

**Solution** : Utiliser `Query<&Window>` pour centrer sur la résolution réelle.

**Fichier** : `src/ui/engine.rs`

---

## 5. [UX] Icônes d'items dans l'inventaire

**Problème** : `inventory_grid.rs` → slots sans image.

**Solution** : Ajouter `ImageNode` avec texture depuis `TextureCache`.

**Fichier** : `src/ui/components/inventory_grid.rs`

---

## 6. [UX] Workbench : craft manuel avec progression

**Problème** : `production.rs:assembler_tick` craft automatiquement sans intervention joueur.

**Comportement attendu** :
1. Joueur sélectionne une recette dans le panneau
2. Maintient un bouton (clic ou touche)
3. Barre de progression avance
4. Après quelques secondes, objet crafté

**Solution** : Nouveau système de craft manuel pour le workbench, distinct de `assembler_tick` automatique. UI avec bouton maintenable + barre de progression.

**Fichier** : `src/player/crafting.rs` (nouveau système) + UI dans `panel_crafting.toml`
