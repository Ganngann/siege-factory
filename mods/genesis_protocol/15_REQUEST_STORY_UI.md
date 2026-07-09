# Request 15 — Système Data Pad + Objectif HUD + Toast dismiss

## Composant A — Objectif HUD (visible en permanence)

**Position :** En haut à gauche de l'écran, sous la barre d'info du joueur.

```
 ┌─────────────────────────────────────────┐
 │ OBJECTIF                                   │
 │ Réparer la capsule Genesis                 │
 │  > Fabriquer un marteau à l'Établi         │
 └─────────────────────────────────────────┘
```

**Données :** `data/objectives.toml` (12 objectifs, triggers par tutorial_step / tier_unlocked)

**Rendu :**
- Fond `rgba(0,0,0,0.3)`, bordure gauche 2px blanche ou colorée
- Police 14px blanche
- Première ligne : objectif en cours (titre)
- Deuxième ligne (indentée) : sous-objectif actuel (étape tutoriel), préfixé de `>`
- Le tout ne doit **pas** disparaître — visible tant que non remplacé

**Fichier à modifier :** `src/player/objective.rs` (rendu UI actuel à corriger ou compléter)

---

## Composant B — Data Pad (histoire consultable)

**Contexte :** Les logs narratifs (`story/logs.toml`, 9 entrées) sont débloqués un par un via les tiers de capsule. Actuellement ils ne sont visibles qu'une fois (toast perdu au clic). Le Data Pad permet de les relire à tout moment.

**Emplacement d'ouverture :** Pas de touche dédiée. Le Data Pad est un onglet dans le **panneau de la capsule** (ouvert par clic gauche sur le genesis_ark).

**Fichier à créer :** `src/economy/data_pad.rs`

### Apparence du panneau capsule complet

```
┌───────────── CAPSULE GENESIS ────────────────┐
│                                                │
│  ● Tier 0 : Déblayage (complété)              │
│  ● Tier 1 : Réveil (complété)                 │
│  ○ Tier 2 : Étincelle                         │
│    Items requis :                             │
│    [▰▰▰▰▰▰▰▰▰▰] Pièces de Fer    5/5   ✓     │
│                                                │
│  [E] Donner les items requis                   │
│                                                │
│  ┌────────────── [📖] Data Pad ───────────────┐│
│  │ ● Déblayage              "Le marteau       ││
│  │ ● Réveil                 frappe les        ││
│  │ ○ Étincelle               panneaux..."     ││
│  │ ○ Rouille & Vapeur                         ││
│  │ ○ Fil du Cuivre                            ││
│  │ ○ Pouls                                    ││
│  │ ○ Nanites                                  ││
│  │ ○ Genèse                                   ││
│  │ ○ Premier Souffle                          ││
│  └────────────────────────────────────────────┘│
│                                                │
└────────────────────────────────────────────────┘
```

### Légende

| Symbole | Signification |
|---------|---------------|
| `●` | Log débloqué — cliquable, affiche le texte complet |
| `○` | Log verrouillé — pas cliquable |
| `>` | Log actuellement sélectionné pour lecture |

### Texte complet affiché (quand on clique sur un ●)

```
┌─ Déblayage ──────────────────────────────────┐
│                                                │
│  "Le marteau frappe les panneaux d'accès.     │
│   La rouille cède — et le manche avec..."      │
│                                                │
│  Tier 0 — Phase 0 (Réveil)                     │
└────────────────────────────────────────────────┘
```

- Titre en gras, taille 16px
- Texte en italique (guillemets), taille 14px
- Pied : `Tier X — NomPhase` en gris clair

### Interaction

| Action | Résultat |
|--------|----------|
| Clic sur `●` (débloqué) | Affiche le texte complet en bas du Data Pad |
| Clic sur `○` (verrouillé) | Ne fait rien (ou toast "Tier non atteint") |
| Survol d'un log | Fond légèrement plus clair |
| Molette | Défilement si liste trop longue |

### Données consommées

```rust
// ProgressionLogRegistry (déjà dans tiered_structure.rs)
struct ProgressionLogRegistry {
    logs: Vec<LogEntry>,
}

struct LogEntry {
    id: String,
    tier: u32,
    title: String,
    text: String,
}
```

Les logs sont déjà chargés par `ProgressionLogRegistry` et liés aux tiers via `log_id` dans `buildings.toml`. Le Data Pad n'a qu'à lire les logs débloqués via le `CurrentTier` de la capsule.

**Fichiers à modifier :**

| Fichier | Changement |
|---------|------------|
| `src/economy/data_pad.rs` | NOUVEAU — UI du Data Pad (liste + affichage texte) |
| `src/economy/inspect/spawn.rs` | Ajouter la section Data Pad dans `open_capsule_panel` (connecter au `ProgressionLogRegistry`) |
| `src/economy/mod.rs` | Enregistrer le nouveau module |

---

## Composant C — Toast dismiss sécurisé

**Problème actuel :** `dismiss_persistent_toasts` dans `src/core/toast.rs` écoute `MouseButton::Left` global — n'importe quel clic ferme le toast persistant, même si le clic est sur le décor, un bâtiment, le sol, etc.

**Fix :** Le toast persistant ne doit se fermer que sur :
1. **Clic gauche directement sur le toast** (vérifier que l'entité cliquée est un `ToastMessage`)
2. **Touche Espace** (inchangé)

```rust
// Au lieu de :
if !buttons.just_pressed(MouseButton::Left) && !keys.just_pressed(KeyCode::Space) {
    return;
}

// Devenir :
if !keys.just_pressed(KeyCode::Space) {
    return;
}
// ET pour le clic : utiliser un système séparé qui vérifie Interaction::Pressed
// sur l'entité ToastMessage elle-même
```

Alternative plus simple : **Supprimer le dismiss par clic gauche** — ne garder que la touche Espace. Le joueur apprend vite que Espace ferme le toast narratif. Moins de risque de dismiss accidentel.

---

## ✅ Implémentation Rust — terminée

| Fichier | Changement |
|---------|------------|
| `src/player/objective.rs` | HUD positionné haut-gauche, fond semi-transparent, bordure gauche blanche, texte au format `OBJECTIF > ...` |
| `src/economy/inspect/data_pad_ui.rs` | **NOUVEAU** — système `data_pad_select_log` qui met à jour l'affichage au clic |
| `src/economy/ui_components.rs` | **NOUVEAUX** composants `DataPadEntry`, `DataPadFullText`, ressource `DataPadSelected` |
| `src/economy/inspect/spawn.rs` | `open_capsule_panel` réécrit : progression, items requis, **liste des logs cliquable**, texte complet |
| `src/core/toast.rs` | Dismiss par Espace uniquement (clic gauche supprimé) |
| `src/economy/inspect/interaction.rs` | Passage de `DataPadSelected` à `open_capsule_panel` |
| `src/economy/mod.rs` | Enregistrement de `DataPadSelected` + `data_pad_select_log` |
