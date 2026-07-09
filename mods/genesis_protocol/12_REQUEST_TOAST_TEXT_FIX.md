# Request 12 — Fix affichage toasts narratifs

## Problème 1 : Overflow ✅ Résolu

`TextLayout::new(Justify::Center, LineBreak::WordBoundary)` + `flex_wrap: Wrap` — le texte passe à la ligne.

## Problème 2 : Centrage ✅ Résolu

`left: Val::Percent(50.0)` → `left: Val::Auto, right: Val::Auto` + `max_width: 500px` — centré sans déborder.

## Problème 3 : Premier caractère du toast sauté ❌

**Cause :** `src/core/toast.rs` ligne 44 — le slice du préfixe persistant est `&msg[14..]` mais le préfixe `\x00PERSISTENT\x00` ne fait que **12 bytes** (2×`\x00` + 10 lettres = 12). Les 2 premiers caractères du message sont avalés → "Année" devient "née".

**Fix :** Changer `&msg[14..]` en `&msg[12..]`.

```rust
// Ligne 30 — format!("\x00PERSISTENT\x00{}", msg)
//   \x00 = 1, PERSISTENT = 10, \x00 = 1 → total 12

// Ligne 44 — ACTUEL (FAUX) :
let text = if persistent { &msg[14..] } else { &msg };

// CORRECTION :
let text = if persistent { &msg[12..] } else { &msg };
```

## Problème 4 : Accents non rendus ❌

Les caractères accentués (é, è, etc.) apparaissent comme des carrés.

**Analyse :** `TextFont { font: font.0.clone().into(), ... }` — ligne 52-55 du toast utilise `GameFont`. Le problème vient peut-être de la police `GameFont` chargée par le jeu. Si `GameFont` utilise une police qui ne contient pas les glyphes accentués (ex: une variante subset de FiraSans), les accents ne s'affichent nulle part dans le jeu. À vérifier dans `src/core/game_font.rs` et quel fichier `.ttf` est chargé.
