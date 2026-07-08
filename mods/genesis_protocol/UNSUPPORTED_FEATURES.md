# Genesis Protocol — Fonctionnalités non supportées

Ce document liste les éléments du design original qui **ne peuvent pas être implémentés** via le système de modding TOML seul. Ces fonctionnalités nécessiteraient des modifications du code Rust du jeu de base (`src/`).

---

## 1. Système d'Outils (Phase 0)

**Design original** : stone_axe, stone_pickaxe, stone_blade, hammer — outils durables craftés à la main avec des propriétés (coupe des arbres, minage, etc.).

**Blocage technique** : Le jeu n'a pas de système d'équipement joueur. Le joueur n'a pas d'inventaire d'outils, pas de notion d'outil équipé, pas de durabilité. Le minage est géré par la touche E + clic.

**Solution possible** (code Rust) : Ajouter un composant `EquippedTool` au joueur, un système de `ToolDef` registry, et modifier le système de minage pour vérifier l'outil équipé.

---

## 2. Fluides et Transport par Tuyaux (Phase 2)

**Design original** : Eau pompée transportée par tuyaux, vapeur produite par chaudière, circuits de refroidissement.

**Blocage technique** : Le jeu ne gère que des ressources discrètes (items). Pas de système fluide avec volumes, pression, débit. Les tuyaux comme mécanique de transport n'existent pas (seuls les belts existent).

**Solution possible** (code Rust) : Ajouter un système de fluides avec réservoirs, tuyaux, pompes. Ou à minima ajouter un type `FluidInventory` parallèle à `Inventory`.

---

## 3. Générateur Vapeur (combustion multi-ressource, Phase 2)

**Design original** : Water + Coal → Steam + Energy. Le générateur consomme de l'eau ET du charbon pour produire de la vapeur et de l'électricité.

**Blocage technique** : Le `BurnerGenerator` existant ne brûle qu'un seul type de carburant (dans son inventaire). Il ne peut pas consommer deux ressources simultanément.

**Solution possible** (code Rust) : Créer un nouveau type de générateur avec une recette d'input/output qui transforme water+coal→steam+energy, couplé à production d'énergie.

---

## 4. Foreuse Profonde (ressources infinies, Phase 5)

**Design original** : `Deep Core Drill` — extrait des minerais en profondeur de manière illimitée.

**Blocage technique** : Le système de gisements (`requires_deposit`) lie un bâtiment à un gisement. Mais les gisements ne sont pas infinis (même avec `infinite=true`, ils sont liés à des tuiles spécifiques).

**Solution possible** (code Rust) : Ajouter un flag `infinite_deposit` ou un composant spécial pour la foreuse profonde.

---

## 5. Compacteur (compression 4:1, Phase 5)

**Design original** : Compresse 4 items en 1 (ratio 4:1). Mécanique de compression automatisée.

**Blocage technique** : Il n'existe pas de mécanique "4 items entrent, 1 sort" qui ne soit pas une recette. Le compacteur ne pourrait fonctionner que comme une recette normale.

**Solution possible** (code Rust) : Ajouter un système de transformation d'items dans l'inventaire, déclenché périodiquement.

---

## 6. Capsule — Compte à Rebours Final

**Design original** : Quand les 4 composants ultimes sont insérés, un compte à rebours de 60s s'affiche, puis la capsule s'illumine.

**Blocage technique** : Le système `tiered_structure` avance immédiatement le tier quand les items sont livrés. Pas de séquence différée, pas d'animation de compte à rebours.

**Solution possible** (code Rust) : Ajouter un état `FinalCountdown` à la capsule avec timer et événement de fin de jeu.

---

## 7. Biomes / Environnement Variable

**Design original** : Différentes zones (clairière, ruines, forêt dense) avec ressources spécifiques.

**Blocage technique** : Pas de système de biomes. Les décorations sont aléatoires et uniformes.

**Solution possible** (code Rust) : Ajouter un système de régions/biomes avec des tables de ressources et décoration par biome.

---

## 8. Découvertes Débloquant des Bâtiments (chaînage complet)

**Design original** : Chaque phase se débloque en craftant un objet clé (ex: "craft stone_pickaxe → débloque phase 1").

**Blocage technique** : Les découvertes ne débloquent que des recettes, pas des buildings directement. Les buildings utilisent `requires_discovery` qui vérifie si une recette est dans le `GlobalArchive`, ce qui permet un contournement fonctionnel.

**Solution partielle** : Fonctionnel avec le système actuel (via recettes-clef), mais moins fluide que le design original. Le joueur doit produire suffisamment d'items dans un bâtiment pour déclencher la découverte qui débloque la recette-clef de la phase suivante.

---

## 9. Outils Pierre (supprimés du mod)

Les recettes `stone_pickaxe` et `stone_blade` (présentes dans les starter_recipes) produisent des items sans utilité mécanique car le système d'outils n'existe pas. Elles sont conservées comme items « décoratifs » / de lore uniquement.

---

## Résumé

| # | Fonctionnalité | Priorité | Complexité Rust estimée |
|---|---------------|----------|------------------------|
| 1 | Système d'outils | Haute | Moyenne (∼200 lignes) |
| 2 | Fluides/tuyaux | Haute | Élevée (∼500+ lignes) |
| 3 | Générateur multi-fuel | Moyenne | Faible (∼50 lignes) |
| 4 | Foreuse infinie | Basse | Faible (∼30 lignes) |
| 5 | Compacteur | Basse | Faible (∼50 lignes) |
| 6 | Compte à rebours final | Haute | Moyenne (∼150 lignes) |
| 7 | Biomes | Basse | Élevée (∼400+ lignes) |
| 8 | Chaînage découvertes | — | Déjà contourné |
