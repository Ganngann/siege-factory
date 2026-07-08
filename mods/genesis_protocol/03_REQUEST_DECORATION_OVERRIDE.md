# Request 03 — Pouvoir surcharger les décorations à tous niveaux

## Contexte

Actuellement, les `[[decorations]]` sont des arrays TOML qui se concatènent entre les mods. Un mod ne peut pas remplacer les décorations définies par `mods/base/`. On voudrait pouvoir :

1. **Surcharger par `kind`** — si deux mods définissent `[[decorations]] kind = "tree"`, les paramètres du dernier mod chargé remplacent ceux du premier (au lieu d'ajouter une seconde entrée)
2. **Pouvoir assigner des textures PNG aux décorations** (pas seulement des formes procédurales `shape`/`color`). Actuellement limité à `triangle`/`circle`/`square`.

## Détail

### 1. Surcharge par `kind`

Permettre à un mod de redéfinir complètement une décoration existante :

```toml
[[decorations]]
kind = "tree"
min_size = 4
max_size = 8
density = 0.002
color = "#1A2A1A"
shape = "triangle"
```

Si le mod est chargé après `mods/base/`, cette entrée remplace l'arbre de base au lieu de l'ajouter.

### 2. Support de texture PNG

Ajouter un champ `texture_stem` optionnel aux décorations. Si présent, la décoration utilise l'image `{texture_stem}_base.png` (cherchée dans les dossiers textures du mod ou de `assets/textures/`) au lieu de `shape`+`color`.

```toml
[[decorations]]
kind = "tree"
min_size = 4
max_size = 8
density = 0.002
texture_stem = "genesis_dead_tree"   # <-- nouveau champ optionnel
shape = "triangle"                    # fallback si pas de texture_stem
color = "#1A2A1A"                     # fallback si pas de texture_stem
```

### 3. Option alpha/opacité

Ajouter un champ `opacity` (0.0–1.0) aux décorations pour pouvoir les rendre semi-transparentes, permettant de les fondre dans le décor.

```toml
[[decorations]]
kind = "rock"
min_size = 2
max_size = 3
density = 0.001
color = "#444444"
opacity = 0.4      # <-- nouveau champ optionnel
shape = "circle"
```

## Résultat attendu

Un mod peut :
- remplacer complètement l'apparence d'une décoration sans modifier les fichiers du jeu de base
- utiliser des textures personnalisées pour les décorations (arbres morts, ruines spécifiques au mod)
- rendre les décorations plus ou moins visibles via l'opacité
- rendre les ressources minables visuellement distinctes des décorations non-interactives
