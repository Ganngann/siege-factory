# Blocage 04 — Excavation Rig sans rôle + distribution des dépôts

## 1. Excavation Rig — aucun rôle défini

**Design** (TECH_TREE.md Phase 5) : `Excavation Rig — 8 steel + 4 motor + 2 machine_frame — Terrain massif`

**Problème** : Le bâtiment existe dans `buildings.toml` mais n'a :
- Pas de `recipe_categories`
- Pas de `production`
- Pas de `combat`
- Juste une consommation d'énergie (20W)

Aucune mécanique de terrassement/nivellement n'existe dans le jeu. Impossible de lui donner un rôle via les TOML seuls.

**Solutions possibles** (code Rust) :
- Mécanique de « clear terrain » : enlève les décorations dans un rayon
- Mécanique de « zone excavée » : crée un trou/plateforme constructible
- Supprimer du design et du mod

## 2. Distribution des dépôts miniers — risque d'épuisement

**Notre `map_config.toml`** définit :
```toml
[deposits.distribution]
scrap_metal = 40
wood = 30
stone = 20
clay = 10
```

Cela écrase probablement la distribution du jeu de base (`iron_ore=50, copper_ore=35, coal=15`).

**Conséquence** : `iron_ore`, `copper_ore`, `coal` ne spawnent que dans les dépôts de départ (starting_area.structures). Une fois épuisés, plus aucun moyen d'en obtenir — sauf à avoir implémenté `infinite_extraction` sur `manual_miner`, ce qui n'est pas le cas.

**Solutions possibles** :
- Ajouter `infinite_extraction = true` sur `manual_miner` (si applicable)
- OU fusionner les distributions : inclure `iron_ore`, `copper_ore`, `coal` dans notre distribution
- OU demander au moteur de merger (pas de scatter) les distributions entre mods
