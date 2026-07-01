# Roadmap — Siege Factory

## Légende

- [x] = fait
- [-] = partiel / implémenté différemment
- [ ] = à faire
- Durée estimée en sessions de dev (1 session = ~2-4h)

## M1 — Squelette ✅ (livré)

- [x] Projet Bevy, structure des dossiers
- [x] Grille tuilée 20×15, rendu damier
- [x] Machine à états : Loading → Playing → GameOver
- [x] Clic sur tuile → highlight
- [x] 22 tests unitaires + 1 test d'intégration + proptest

## M2 — Économie ✅ (livré)

- [x] ResourceId (Ore, Ammo, Energy)
- [x] ResourceRegistry depuis `data/resources.toml`
- [x] Inventory component (saturating math)
- [x] 6 gisements Ore placés sur la carte
- [x] HQ spawn avec 20 Ore de départ
- [x] Miner : coût 10 Ore, produit 1 Ore/2s
- [x] Assembler : coût 15 Ore, consomme 3 Ore → 1 Ammo/2s
- [x] UI : compteur Ore/Ammo/Energy, palette construction (touches 1-5)
- [x] Tests : Inventory pur (add, remove, overflow, underflow)

## M3 — Constructions ✅ (livré)

- [x] BuildingRegistry + BuildingDef depuis `data/buildings.toml`
- [x] Palette de construction (touches 1-5, Escape désactive)
- [x] Placement : clic → building si terrain libre + coût payé (déduit du HQ)
- [x] 5 bâtiments : Miner, Assembler, Belt, Wall, Turret
- [ ] Tests placement/coûts → **à écrire**

## M4 — Ennemis & combat ✅ (livré)

- [x] WaveState : timer, difficulté croissante (HP = 20 + (wave-1)*5)
- [x] BFS pathfinding sur grille 20×15
- [x] Ennemis marchent vers HQ, contournent les murs
- [x] Turret (coût 20 Ore + 5 Ammo) : tire 5 dmg/1s, range 4 tuiles
- [x] HQ HP (100), game over si détruit
- [x] Ennemis spawnent aux bords de la carte
- [x] Soldier (key 6, 10 Ore) : attaque auto 8 dmg/1s, range 3 tuiles
- [x] Worker (key 7, 5 Ore) : récolte automatique des gisements
- [x] Nettoyage des entités au restart (enemies + soldiers + workers)
- [x] Écran GameOver : vague survécue, touche R pour restart
- [x] Wave auto-advance après 15s sans ennemis
- [x] Formes géométriques (Mesh2d) pour toutes les entités
- [x] HUD compteur de vagues (top-right, live Query)
- [ ] Tests pathfinding/spawn/combat → **à écrire**

## M4.1 — Conditions de victoire ✅ (livré)

- [x] `WIN_WAVES = 10` constant
- [x] Écran de victoire (vert "VICTORY" si wave > WIN_WAVES)
- [x] Compteur de vagues affiche `Wave N/10`
- [ ] Mode survival infini (sans win condition, boucle infinie) → **optionnel**
- [ ] Mode campagne distinct (menu choix campagne/survival) → **optionnel**

## M5 — Data-driven ✅ (livré)

- [x] ResourceRegistry depuis `data/resources.toml`
- [x] BuildingRegistry + CombatStats depuis `data/buildings.toml`
- [x] EnemyRegistry depuis `data/enemies.toml`
- [x] RecipeRegistry depuis `data/recipes.toml`
- [x] `MapConfig` resource depuis `data/map_config.toml` (tile_size, grille, dépôts, HQ)
- [x] `WaveConfig` resource depuis `data/waves.toml` (win_waves, spawn, timers, HP)
- [x] `UnitConfig` resource depuis `data/units.toml` (soldat/worker stats, coûts, dégâts)
- [x] Plus aucune constante hardcodée — tout dans `data/*.toml`
- [x] Build WASM : `build_wasm.ps1` + `wasm-bindgen` OK, `web/` prêt
- [ ] Déploiement web (itch.io / GitHub Pages)
- [ ] Playtest en ligne + ajustements équilibrage

## M6 — Évolution RTS (2-3 sessions restantes)

- [x] Unités mobiles : Soldier (key 6, attaque auto), Worker (key 7, récolte automatique)

### M6.1 — Logistique ✅ (livré)
- [x] Direction enum (East/North/West/South, Default=East) avec rotation (touche R)
- [x] Belt component + chaînage avant/arrière (`trace_belt_output`, `trace_chain`, `find_input_source`, `trace_chain_back`)
- [x] Miner → belts → bâtiments connectés (Ore circule dans la chaîne)
- [x] Assembler : lit depuis belts entrants, écrit vers belts sortants
- [x] Rendu ASCII des belts (`> ^ < v`, font_size 24)
- [x] Palette build mode : indicateur de direction + hint "R: rotate"
- [x] Items sur belts : billes colorées (Ore=jaune, Ammo=rouge) qui circulent le long des chaînes

### M6.2 — Worker automation ⚠️ (partiel — 1 session restante)
- [x] Worker : récolte automatique le gisement Ore le plus proche (1 Ore/3s dans le HQ)
- [x] Dépôt se déplète et se déspawn quand épuisé
- [ ] Worker : construction automatique du bâtiment le plus proche non construit
- [ ] File d'ordres (build queue)

### M6.3 — Sélection & ordres (1-2 sessions)
- [ ] Sélection box (drag-select)
- [ ] Ordres : déplacement (clic droit), attaque ciblée, récolte
- [ ] Gestion des groupes

### M6.4 — Arbre de technologies (1 session)
- [ ] TechTree chargé depuis `data/techs.toml`
- [ ] Niveaux de bâtiments (Miner II, Turret II, etc.)
- [ ] Déblocages progressifs

### M6.5 — Fog of war (1 session)
- [ ] Vision limitée autour des unités et bâtiments
- [ ] Découverte progressive de la carte

## M7 — Combat visible ✅ (livré)

- [x] **Projectiles visuels** : cercles volant de turret/soldat vers ennemi (homing, mesh partagé)
- [x] **Items sur belts** : billes colorées (Ore=jaune, Ammo=rouge) qui transitent tile→tile le long des chaînes
- [ ] Tests combat (projectiles, dégâts, interactions) → **à écrire**

## M8 — Contrôle direct & variété (2-3 sessions)

- [ ] **Barre de construction UI** : boutons cliquables avec coûts (fin des touches 1-5)
- [ ] **Sélection + clic droit** : clic sur unité/bâtiment, clic droit déplacement/attaque
- [ ] **Auto-route belts** : Shift+clic source → destination → BFS + placement automatique des belts
- [ ] **Auto-build** : workers construisent les bâtiments placés dans la file
- [ ] **Spawn tanks + boss** (déjà définis dans `data/enemies.toml`)
- [ ] **Équilibrage** : coûts, HP, timing via TOML tweaks
- [ ] Tests placement/coûts + pathfinding/spawn/combat + combat

## M9 — Multijoueur (4-6 sessions)

- [ ] Simulation déterministe (GameSeed, FrameNumber)
- [ ] NetworkId sur entités clés
- [ ] Command pattern (Events → inputs réseau)
- [ ] Connexion P2P (quinn)
- [ ] Synchronisation d'état
- [ ] Mode PvP (écran partagé d'abord)
- [ ] Serveur notaire (matchmaking, ELO)
- [ ] Anti-triche (hashs, replay)

## M10 — Polissage final & Release (2-3 sessions)

- [ ] Menu principal (titre, boutons Jouer/Options/Quitter)
- [ ] Textures et sprites (remplacer les carrés de couleur)
- [ ] Animations (ennemis, unités, bâtiments)
- [ ] Intégration Steamworks (steamworks-rs)
- [ ] Build optimisé (LTO, profile release)
- [ ] Tuning perf, profiling
- [ ] Packaging Windows/Mac/Linux
- [ ] Déploiement web (itch.io / GitHub Pages)

## Dépendances

```
M1 → M2 → M3 → M4 → M4.1 → M5 ─→ M6.x → M7 → M8 → M9 → M10
```

- **M4.1** fait en parallèle de M5 (déjà livré)
- **M5** terminé — plus de blocage data-driven
- **M6.x** : M6.1 et M6.2 (récolte) livrés ; reste auto-build, sélection, ordres
- **M7** livré — combat visible (projectiles + belts items)
- **M9** nécessite M8 terminé (sélection + ordres nécessaires au PvP)
- **M10** peut commencer dès que le build web est déployé
