# Genesis Protocol — Fonctionnalités non supportées

Ce document liste les éléments du design original qui **ne peuvent pas être implémentés** via le système de modding TOML seul. Ces fonctionnalités nécessiteraient des modifications du code Rust du jeu de base (`src/`).

Les sections marquées ✅ sont désormais **implémentées dans le code Rust** et utilisables via les fichiers TOML.

---

## 1. Système d'Outils (Phase 0) ✅

**Design original** : stone_axe, stone_pickaxe, stone_blade, hammer — outils durables craftés à la main avec des propriétés (coupe des arbres, minage, etc.).

**Solution implémentée** : Système passif sans équipement ni durabilité. Les outils sont des items craftés une fois, placés dans l'inventaire. Quand le joueur mine (touche E), le système détecte automatiquement si son inventaire contient un outil compatible avec la ressource du dépôt. Si oui, le minage est 2× plus rapide (ou configurable via `mine_interval_mult`).

**Usage TOML** — Créer `mods/genesis_protocol/data/tools.toml` :

```toml
[tools.stone_pickaxe]
allowed_resources = ["stone", "clay", "coal", "iron_ore", "copper_ore"]
mine_interval_mult = 0.5

[tools.stone_axe]
allowed_resources = ["wood", "scrap_metal"]
mine_interval_mult = 0.5

[tools.stone_blade]
allowed_resources = ["plant_fiber"]
mine_interval_mult = 0.5

[tools.hammer]
allowed_resources = []
mine_interval_mult = 1.0
```

Les ressources dans `allowed_resources` correspondent aux types de dépôts que l'outil accélère. Le `mine_interval_mult` est un multiplicateur du temps de minage (0.5 = 2× plus rapide).

Ajouter les recettes dans `recipes.toml` :

```toml
[recipes.stone_pickaxe]
category = "hand_crafting"
craftable_in = ["workbench"]
input = { stone = 2, wood = 1 }
output = { stone_pickaxe = 1 }
time_sec = 5.0
```

**Comportement** :
- Le joueur craft l'outil une fois → reste dans l'inventaire
- Le système détecte automatiquement l'outil dans l'inventaire → bonus de vitesse sans équipement manuel
- L'outil n'est jamais consommé, pas de durabilité
- Si plusieurs outils compatibles, le premier trouvé est utilisé

**Le hammer** : pas besoin de système d'outils. Il suffit de le définir comme une ressource craftable + une découverte déclenchée par son craft (via le système de découverte existant). Le changement de la découverte Phase 0 (du campfire vers le hammer) se fait en TOML pur.

---

## 2. Fluides et Transport par Tuyaux (Phase 2) ❌

**Design original** : Eau pompée transportée par tuyaux, vapeur produite par chaudière, circuits de refroidissement.

**Blocage technique** : Le jeu ne gère que des ressources discrètes (items). Pas de système fluide avec volumes, pression, débit. Les tuyaux comme mécanique de transport n'existent pas (seuls les belts existent).

**Solution possible** (code Rust) : Ajouter un système de fluides avec réservoirs, tuyaux, pompes. Ou à minima ajouter un type `FluidInventory` parallèle à `Inventory`.

---

## 3. Générateur Vapeur (combustion multi-ressource, Phase 2) ✅

**Design original** : Water + Coal → Steam + Energy. Le générateur consomme de l'eau ET du charbon pour produire de la vapeur et de l'électricité.

**Solution implémentée** : `RecipeGenerator` — un composant hybride qui fonctionne comme un `Assembler` (recette entrée/sortie) **et** produit de l'énergie électrique dans le grid. Il remplace `BurnerGenerator` pour les bâtiments qui ont à la fois `default_recipe` et `fuel_burn_interval`.

**Usage TOML** (déjà configuré dans `buildings.toml`) :

```toml
[buildings.steam_generator]
name = "Générateur Vapeur"
# ...
default_recipe = "steam_produce"    # Recette à exécuter
power_consumption = 5.0             # Consommation électrique (démarrage/pompe)
power_generation = 30.0             # Production électrique nette
fuel_burn_interval = 2.0            # Intervalle de « combustion » (synonyme de production_interval)
```

La recette associée (`recipes.toml`) :

```toml
[recipes.steam_produce]
category = "steam"
craftable_in = ["steam_generator"]
input = { water = 2, coal = 1 }
output = { steam = 3, energy = 2 }
time_sec = 4.0
```

**Comportement** :
- Le générateur consomme l'eau et le charbon de son inventaire selon la recette
- Il produit de la vapeur et de l'énergie (items) + alimente le réseau électrique
- Il ne tourne que s'il est actif **et** alimenté (si `power_consumption > 0`)
- S'il manque des ressources, la production électrique tombe à 0

---

## 4. Foreuse Profonde (ressources infinies, Phase 5) ✅

**Design original** : `Deep Core Drill` — extrait des minerais en profondeur de manière illimitée.

**Solution implémentée** : Nouveau champ `infinite_extraction: bool` dans les définitions de bâtiments. Quand activé, le dépôt n'est pas consommé au placement.

**Usage TOML** — ajouter sur le bâtiment dans `buildings.toml` :

```toml
[buildings.deep_core_drill]
name = "Foreuse Profonde"
# ...
requires_deposit = true
infinite_extraction = true   # ← nouveau flag
production = { resource = "ore", interval_sec = 1.0 }
```

**Comportement** : La foreuse se place sur un dépôt, mais le dépôt reste à l'écran et conserve sa quantité. Le mineur continue d'extraire indéfiniment.

---

## 5. Compacteur (compression 4:1, Phase 5) ✅

**Design original** : Compresse 4 items en 1 (ratio 4:1). Mécanique de compression automatisée.

**Solution implémentée** : Nouveau type de bâtiment `"compactor"` avec le composant `Compactor` qui détecte les items en inventaire, en consomme N pour en produire 1 compressé (`{resource}_compressed`).

**Usage TOML** (déjà configuré dans `buildings.toml`) :

```toml
[buildings.compactor]
name = "Compacteur"
# ...
inventory_capacity = 32
compactor_ratio = 4        # nombre d'items consommés (défaut: 4)
compactor_interval = 2.0   # secondes entre chaque compression (défaut: 2.0)
```

**Ressources compressées** — définir les versions compressées dans `resources.toml` :

```toml
[resources.iron_ore_compressed]
name = "Minerai de Fer Comprimé"
max_stack = 999
color = "#B35F33"
```

**Comportement** : Quand l'inventaire contient ≥ 4 unités d'une même ressource, le compacteur en consomme 4 et produit 1 unité de `{ressource}_compressed`.

---

## 6. Capsule — Compte à Rebours Final ✅

**Design original** : Quand les 4 composants ultimes sont insérés, un compte à rebours de 60s s'affiche, puis la capsule s'illumine.

**Solution implémentée** : Nouveau système `FinalCountdown` qui démarre automatiquement quand le dernier palier de la capsule (`genesis_ark`) est livré. La ressource `FinalCountdown` ticke 60s → toasts aux paliers → transition vers l'état `Win`.

**Usage TOML** (déjà configuré) — le dernier tier de `genesis_ark` dans `buildings.toml` :

```toml
[[buildings.genesis_ark.tiers]]
required_items = { neural_interface = 1, synthetic_heart = 1, genome_sequence = 1, fusion_core = 1 }
unlock_recipes = []
log_id = "genesis_final"
texture = "genesis_capsule_t7"
```

**Comportement** :
- Le joueur livre les 4 composants ultimes à la capsule (touche E)
- Le tier avance normalement (CurrentTier + 1, sprite mis à jour)
- Comme c'est le dernier tier (current + 1 == tiers.len()) **et** que l'entité a le composant `Capsule`, le compte à rebours de 60 secondes démarre
- Toasts aux paliers 30s, 10s, 5s, 3s, 2s, 1s
- À 0 : toast final « La capsule s'illumine. Un premier souffle. » → transition vers `GameState::Win`

---

## 7. Biomes / Environnement Variable ❌

**Design original** : Différentes zones (clairière, ruines, forêt dense) avec ressources spécifiques.

**Blocage technique** : Pas de système de biomes. Les décorations sont aléatoires et uniformes.

**Solution possible** (code Rust) : Ajouter un système de régions/biomes avec des tables de ressources et décoration par biome.

---

## 8. Découvertes Débloquant des Bâtiments (chaînage complet)

**Design original** : Chaque phase se débloque en craftant un objet clé (ex: "craft stone_pickaxe → débloque phase 1").

**Blocage technique** : Les découvertes ne débloquent que des recettes, pas des buildings directement. Les buildings utilisent `requires_discovery` qui vérifie si une recette est dans le `GlobalArchive`, ce qui permet un contournement fonctionnel.

**Solution partielle** : Fonctionnel avec le système actuel (via recettes-clef), mais moins fluide que le design original. Le joueur doit produire suffisamment d'items dans un bâtiment pour déclencher la découverte qui débloque la recette-clef de la phase suivante.

---

## 9. Outils Pierre ✅ résolu

Le système d'outils passif (section 1) donne un usage mécanique à `stone_pickaxe`, `stone_axe` et `stone_blade`. Voir section 1 pour la configuration TOML.

---

## Résumé

| # | Fonctionnalité | Statut | Priorité |
|---|---------------|--------|----------|
| 1 | Système d'outils | ✅ Implémenté (ToolRegistry passif) | — |
| 2 | Fluides/tuyaux | ❌ Non implémenté | Haute |
| 3 | Générateur multi-fuel | ✅ Implémenté (RecipeGenerator) | — |
| 4 | Foreuse infinie | ✅ Implémenté (infinite_extraction) | — |
| 5 | Compacteur | ✅ Implémenté (Compactor) | — |
| 6 | Compte à rebours final | ✅ Implémenté (FinalCountdown) | — |
| 7 | Biomes | ❌ Non implémenté | Basse |
| 8 | Chaînage découvertes | ✅ Contourné via recettes | — |
| 9 | Outils Pierre décoratifs | ✅ Résolu (système outil passif) | — |
