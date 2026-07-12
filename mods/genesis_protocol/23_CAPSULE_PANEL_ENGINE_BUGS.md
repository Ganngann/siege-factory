# 23 — Capsule panel: engine bugs restants

## Contexte
`panel_capsule.toml` est chargé (960×720), les composants TOML sont appelés.
Mais le rendu ne correspond pas à `docs/22_UI_DATA_DRIVEN.md`.

## Bugs bloquants

### 1. Wireframe : fond opaque cache les formes

**Fichier :** `src/ui/components/wireframe.rs:135-148`

`update_capsule_wireframe_system` change `BackgroundColor` du conteneur wireframe
avec la couleur du statut `power` (rouge `#ff3333` au tier 0).

Le conteneur devient un rectangle rouge massif — les formes (ellipse, rect, hline, vline)
sont dessinées par-dessus avec `BackgroundColor` ou `BorderColor` mais sont masquées.

**Fix attendu :**
- Le conteneur wireframe doit avoir `BackgroundColor(Color::NONE)` ou un fond très sombre
  fixe, jamais la couleur du statut.
- Seules les formes individuelles doivent être colorées par le statut `power` (leur
  `color_key` doit être interprété comme une clé de registre, pas un hex fixe).
- Ou alternativement : `update_capsule_wireframe_system` devrait itérer les formes
  enfants et changer leur couleur, pas celle du fond.

---

### 2. Scanlines : pas de vrai effet CRT

**Fichier :** `src/ui/components/overlay.rs`

`effect = "scanlines"` crée juste un calque semi-transparent noir.
Aucun motif de lignes n'est dessiné.

**Fix attendu :**
- Soit un motif répété (via `BorderImage` ou `StyleSheet`),
- Soit un shader personnalisé.
- Actuellement ça ne sert à rien visuellement.

---

### 3. LED statique (non animée)

**Fichier :** `src/ui/components/frame.rs:47-61`

La LED du paramètre `led = "red"` est un carré 8×8 statique.

**Fix attendu :**
- La LED doit pulsater (lentement en mode alerte).
- Ou clignoter quand le statut power est rouge.

---

### 4. Typographie uniforme

Tout le texte utilise `theme.font_size_small` (≈11px).
Aucune hiérarchie visuelle entre titres, clés, valeurs, logs.

**Fix attendu :**
- Titres de `frame` / `alert_header` : `theme.font_size_medium` (14px)
- Clés (`key_value`) : `theme.font_size_small` bold (12px bold)
- Valeurs (`key_value`) : `theme.font_size_small` normal (12px)
- Logs list (`data_list`) : `theme.font_size_small` (11px)
- Log text (`data_text`) : `theme.font_size_body` (12px, wrap)

---

### 5. Section JOURNAL DE BORD invisible

La section `frame "JOURNAL DE BORD"` est bien dans le TOML
(3e `[[sections]]`), mais elle n'apparaît pas dans le rendu.

Causes possibles :
- **Overflow clip** : `spawn_window` fixe `height: Val::Px(720)` et
  `overflow: Overflow::clip()`. Si les sections précédentes dépassent,
  le journal est coupé. Vérifier les hauteurs cumulées.
- **data_list / data_text ne rendent rien** : `populate_data_list` utilise
  `Added<DataList>` mais ne s'exécute peut-être pas dans le bon set.
  Vérifier que `PlayingSystems` est actif quand le panneau capsule s'ouvre.
- **populate_data_list ne trouve aucun log débloqué** : maintenant que
  `system_boot` est auto-débloqué (`tiered_structure.rs:52`), ça devrait
  marcher, mais vérifier que `ProgressionLogRegistry` est accessible.

**Fix attendu :**
- Supprimer `overflow: clip` du window root OU rendre le panneau scrollable.
- Vérifier que `populate_data_list` et `animation_tick_system` sont bien
  exécutés dans un set actif au moment de l'ouverture du panneau.
- Si `data_list` ne peuple pas les boutons, ajouter un log dans le système.

---

### 6. Per-status warning icons

`key_value_list` ne supporte pas d'icône par ligne.
Actuellement j'ai préfixé les clés avec `\u{26a0}` dans le TOML
(contournement).

**Fix attendu :**
- Ajouter un champ optionnel `icon` à chaque item de `key_value_list` :
  ```toml
  { key = "ALIMENTATION", value_key = "capsule.status_power", icon = "warning" }
  ```
- Ou laisser le workaround unicode et passer à autre chose.

---

## Bonus / polish

### 7. Phase list : 8 phases pour 9 tiers

**Fichier :** `src/economy/inspect/interaction.rs:93-103`

Le building a 9 tiers (0-8). Le `phase_names` hardcode 8 entrées (0-7).
Quand tier=8 (final), toutes les phases sont "done" — c'est correct,
mais la dernière phase "SÉQUENCE FINALE" n'a pas de tier dédié.

**Fix :** Ajouter un 9e nom de phase ou laisser tel quel (pas bloquant).

---

## Notes

- La note 0/9 a été corrigée : `interaction.rs:86` → `total_tiers = def.tiers.len() - 1`.
  Maintenant 9 tiers (0-8) s'affichent comme `0/8` → `8/8`.
- Le statut "DÉFAILLANTE (REQ. 200kW)" reste rouge car il n'y a que 8 entrées
  dans le registre de statuts (0-7) ; la tier 8 tombbe sur la 7 via
  `status_for_tier` → correct.
- Tous les composants sont enregistrés dans le registry (`src/ui/mod.rs:39-66`).
- 432 tests passent, `cargo build` OK.
