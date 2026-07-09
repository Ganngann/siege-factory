# Request 06 — Fallback deposit texture quand le PNG est introuvable

## Contexte

Dans `src/map/systems/chunks.rs` lignes 188-216, le rendu des dépôts miniers a un bug :

1. `textures.base.get("wood")` retourne `Some(handle)` même si le PNG n'existe pas
2. Cause : dans `setup_texture_cache` (cache.rs:128-131), `load_png(...).unwrap_or_default()` insère un `Handle::default()` dans la HashMap
3. Résultat : le sprite est créé avec une texture vide/invisible → le dépôt n'a aucun rendu visuel

La branche fallback (cercle coloré, lignes 217-244) n'est jamais atteinte car `get()` retourne toujours `Some` à cause du `unwrap_or_default()`.

## Fix demandé

### Option A (recommandée) : Vérifier si le handle est valide

```rust
// chunks.rs ~ line 188
if let Some(handle) = textures.base.get(&d.resource) {
    if handle.id() == Default::default() {
        // PNG introuvable → fallback cercle coloré
        // ... code fallback actuel (lignes 218-244)
    } else {
        // PNG trouvé → texture normale
        // ... code texture actuel (lignes 189-216)
    }
}
```

### Option B : Ne pas insérer les handles par défaut

```rust
// cache.rs ~ lines 126-132
for stem in &item_stems {
    if let Some(handle) = load_png(&mut images, &mods, s, "base") {
        base.insert(stem.clone(), handle);
    }
}
```

Avec cette approche, si le PNG n'existe pas, la clé n'est pas insérée dans la HashMap, et le `get()` retourne `None` → la branche fallback est correctement atteinte.

L'option B est plus simple et plus propre. Elle corrige aussi le même problème pour les building_stems et enemy_stems (lignes 116-140).

## Résultat attendu

- Si un mod ne fournit pas de PNG pour une ressource, le dépôt s'affiche avec un cercle coloré (basé sur `color` dans resources.toml) au lieu d'être invisible
- Les mods peuvent ne fournir que des PNG pour les ressources qui en ont besoin, les autres héritent du fallback visuel
