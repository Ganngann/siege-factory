# Request 16 — Fix UI panneaux bâtiments (fermeture + sélecteur recettes)

## Constat

Trois bugs bloquants dans l'inspect UI :

1. **Bouton "Recettes" sur l'Établi** : ne fait rien
2. **Bouton "Recettes" sur le Feu de Camp** : ouvre une fenêtre vide. En la fermant, le panneau parent se ferme aussi, puis plus aucune UI ne répond (overlay fantôme)
3. **Clic en dehors du panneau** pour le fermer : ne marche pas

---

## Bug A — Sélecteur de recettes vide

**Fichier mod :** `data/buildings.toml`

**Cause :** Les champs `recipe_categories` manquent sur `workbench` et `campfire`. Le système `recipe_change_system` récupère la liste vide → rien à afficher.

**Fix (moi) :** Ajouter `recipe_categories = ["proto"]` aux deux bâtiments dans le TOML.

---

## Bug B — `close_window_system` despawn tout le panneau (cascade)

**Fichier Rust :** `src/economy/window.rs:221` — `fn close_window_system`

**Cause :** Deux systèmes répondent au `CloseButton` :
- `inspect::close_button_system` (`mod.rs:99`) — gère correctement le sélecteur de recettes
- `window::close_window_system` (`window.rs:221`) — remonte la chaîne parent depuis n'importe quel `CloseButton` et despawn le `WindowRoot`

Quand le joueur clique sur le X du sélecteur de recettes :
1. `close_button_system` → ferme le sélecteur ✅
2. `close_window_system` → remonte du X du sélecteur → trouve `WindowRoot` → **despawn tout le panneau** ❌

**Parade possible :** `close_window_system` ne doit pas toucher aux fenêtres qui sont gérées par `BuildingPanel`. Soit :
- Supprimer `close_window_system` (inutile : `close_button_system` + `overlay_click_system` couvrent tous les cas)
- Ou ajouter un marqueur `ManagedByPanel` que `close_window_system` vérifie avant d'agir

---

## Bug C — Overlay fantôme après fermeture

**Fichier Rust :** `src/economy/window.rs:221` (même fonction)

**Cause :** Quand `close_window_system` despawn le `root` (l'entité `WindowRoot`), l'`overlay` (parent du root, entité avec `PanelOverlay`) **reste vivant**. Le `BuildingPanel.overlay` pointe encore vers cette entité toujours existante. Cette overlay a `Pickable::default()` (qui bloque les clics). Le système `update_ui_blocking` (mod.rs:54) la détecte → `UiIsBlocking = true` → tous les systèmes d'interaction (clic bâtiment, clic dépot, etc.) retournent immédiatement.

**Résultat :** Plus aucun clic UI ne fonctionne jusqu'au redémarrage.

**Fix :** Même parade que le Bug B — ou s'assurer que `close_window_system` appelle `BuildingPanel.close_panel()` au lieu de despawner directement.

---

## Bug D — Overlay click ne ferme pas le panneau

**Fichier Rust :** `src/economy/inspect/spawn.rs` (dans `open_panel`)

**Cause :** `overlay_click_system` (mod.rs:56-95) cherche une entité `PanelModal` pour délimiter la zone du panneau :
```rust
modal_query: Query<(&Node, &GlobalTransform), (With<PanelModal>, Without<PanelOverlay>)>
```
Seul `deposit_panel.rs:87` ajoute `PanelModal` au root. `open_panel` dans `spawn.rs` ne l'ajoute pas → `modal_query.single()` échoue → le système retourne sans rien faire → impossible de fermer un panneau en cliquant à côté.

**Fix :** Ajouter `commands.entity(root).insert(PanelModal);` dans `open_panel` (spawn.rs), juste après `spawn_panel_ui`, exactement comme dans `deposit_panel.rs:87`.

---

## ✅ Implémentation Rust — terminée

| Fichier | Changement |
|---------|------------|
| `src/economy/ui_components.rs` | Nouveau composant `ManagedByPanel` |
| `src/economy/window.rs` | `close_window_system` ignore les fenêtres avec `ManagedByPanel` |
| `src/economy/inspect/spawn.rs` | `open_panel` + `open_capsule_panel` ajoutent `PanelModal` + `ManagedByPanel` au root |

### Bugs corrigés

| Bug | Fix | Statut |
|-----|-----|--------|
| **A** — Recettes vides | Ajouter `recipe_categories` dans le TOML du mod | Côté mod |
| **B** — CloseWindowSystem conflit | `ManagedByPanel` empêche `close_window_system` de toucher aux fenêtres gérées | ✅ |
| **C** — Overlay fantôme | `close_window_system` n'interfère plus → `close_button_system` gère proprement | ✅ |
| **D** — Impossible fermer par clic extérieur | `PanelModal` ajouté sur le root → `overlay_click_system` fonctionne | ✅ |
