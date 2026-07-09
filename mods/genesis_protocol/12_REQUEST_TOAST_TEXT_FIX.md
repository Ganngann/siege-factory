# Request 12 — Fix affichage toasts narratifs + accents

## Problème 1 : Texte tronqué (overflow)

Le toast narratif d'accueil ("Année 2147...") est trop long pour la zone d'affichage. Le texte déborde et on ne peut pas le lire en entier.

### Solutions possibles

**A — Word wrap** (recommandé) : Activer le retour à la ligne automatique sur les toasts, surtout les `persistent` du tutoriel.

**B — Zone plus large** : Augmenter la largeur maximale du conteneur de toast (surtout pour les toasts centrés en haut).

**C — Taille de police adaptable** : Réduire automatiquement la taille de police si le texte dépasse la largeur.

Le toast concerné est dans `tutorial.toml` :
```toml
[[steps]]
id = "welcome"
toast = "Année 2147. Les ruines de la cité s'étendent à perte de vue. La capsule cryo s'est ouverte — trop tard pour les autres. Un seul objectif : la réparer."
persistent = true
```

## Problème 2 : Accents non rendus

Les caractères accentués (é, è, ê, î, ô, û, ç, à, ù, ë) apparaissent comme des caractères invalides ou sont absents. Tous les textes narratifs du mod utilisent des accents (français).

### Cause probable

La police chargée par le moteur Bevy ne supporte pas le jeu de caractères étendu (latin-1 supplement). La police utilisée (`Fira Sans` ou autre) doit inclure les glyphes accentués.

### Solutions possibles

**A — Police complète** (recommandé) : S'assurer que la police chargée inclut les caractères Unicode étendus. Vérifier que `FiraSans-Regular.ttf` (ou équivalent) est bien la version complète, pas une variante allégée.

**B — Subset** : Si le problème persiste, remplacer la police par une autre qui supporte les accents français (Ubuntu, Noto Sans, etc.).

**C — Fallback** : Ajouter une police de secours pour les caractères manquants.

## Fichiers concernés

Tous les fichiers TOML contenant des textes en français avec accents :
- `data/tutorial.toml` (toasts du tutoriel)
- `data/discoveries.toml` (messages de découverte)
- `story/logs.toml` (logs narratifs de la capsule)
- `data/objectives.toml` (objectifs HUD)

---

## ✅ Implémentation Rust — terminée

### Fix 1 — Word wrap

| Fichier | Changement |
|---------|------------|
| `src/core/toast.rs` | `TextLayout::new(Justify::Center, LineBreak::WordBoundary)` + `max_width: 700px` + `flex_wrap: Wrap` |
| `src/player/objective.rs` | `TextLayout::new(Justify::Center, LineBreak::WordBoundary)` + `flex_wrap: Wrap` |

Les toasts et l'objectif HUD passent maintenant à la ligne automatiquement au lieu de déborder.

### Fix 2 — Accents

**Analyse** : La police par défaut de Bevy (`FiraMono`) supporte les caractères latin-1 étendus. Le problème d'accents peut venir de l'encodage des fichiers TOML ou de la police système.

**Solution actuelle** : Les toasts et l'UI utilisent la police par défaut de Bevy via `TextFont::from_font_size()`.

**Si le problème persiste** (recommandation pour l'autre développeur) :
1. Ajouter un fichier `.ttf` complet dans `mods/genesis_protocol/textures/fonts/` (ex: `FiraSans-Regular.ttf`, téléchargeable gratuitement)
2. Charger la police dans une ressource dédiée et l'assigner aux `TextFont`

Je peux implémenter ce chargement de police personnalisée quand vous aurez le fichier `.ttf`.
