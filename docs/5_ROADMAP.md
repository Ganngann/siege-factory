# Roadmap — Siege Factory

## Légende

- [x] = fait
- [-] = partiel
- [ ] = à faire
- Durée estimée en sessions de dev (1 session = ~2-4h)

## Phase 1 — Scaffold TD (jusqu'à socle technique stable)

### Socle — ✅ Livré

- [x] Projet Bevy 0.19, structure des dossiers
- [x] Grille tuilée 20×15, rendu damier
- [x] Machine à états : Menu → Playing → GameOver
- [x] Clic sur tuile → highlight
- [x] Tests unitaires + intégration + proptest

### Économie — ✅ Livré

- [x] ResourceId (Ore, Ammo, Energy)
- [x] ResourceRegistry depuis `data/resources.toml`
- [x] Inventory component (saturating math)
- [x] 6 gisements Ore placés sur la carte
- [x] HQ spawn avec 200 Ore de départ
- [x] Miner / Assembler / Belt / Wall / Turret / Storage / Splitter / Sorter
- [x] Production ticks : mines produisent, assembleurs craftent
- [x] Ceintures : chaînage, items visuels, direction (touche R)
- [x] Belt drag (clic-déplacer pour poser des ceintures)
- [x] UI compteurs Ore/Ammo/Energy + barre construction
- [x] Tests : Inventory pur (add, remove, overflow, underflow)

### Ennemis & Combat — ✅ Livré

- [x] WaveState : timer, difficulté croissante
- [x] BFS pathfinding sur grille 20×15
- [x] Ennemis marchent vers HQ, contournent les murs
- [x] Turret : tir automatique, range, dégâts
- [x] HQ HP (100), game over si détruit
- [x] Ennemis spawnent aux bords
- [x] Soldier / Worker : spawn, attaque auto, récolte auto
- [x] Projectiles visuels (homing)
- [x] Écran GameOver et Victoire
- [x] 10 vagues pour gagner (WIN_WAVES)
- [x] Nettoyage des entités au restart

### Data-driven — ✅ Livré

- [x] ResourceRegistry depuis `data/resources.toml`
- [x] BuildingRegistry + CombatStats depuis `data/buildings.toml`
- [x] EnemyRegistry depuis `data/enemies.toml`
- [x] RecipeBank depuis `data/recipes.toml`
- [x] MapConfig depuis `data/map_config.toml`
- [x] WaveConfig depuis `data/waves.toml`
- [x] UnitConfig depuis `data/units.toml`
- [x] MenuDef depuis `data/menu.toml` (menu arborescent infini)
- [x] KeyBindings depuis `data/keybindings.toml`
- [x] MainMenu depuis `data/main_menu.toml`
- [x] Build WASM : `build_wasm.ps1` + `wasm-bindgen`

### Rendu & UI — ✅ Livré

- [x] Formes géométriques (Mesh2d) pour toutes les entités
- [x] HP bars (colorées par ratio)
- [x] Tile highlight survolée
- [x] Items sur belts (billes colorées)
- [x] Menu arborescent avec scroll, hotkeys dynamiques
- [x] Tooltips au survol des boutons de construction
- [x] Main menu (titre, Play/Options)
- [x] Game over / Victory screen

### Restant dans la Phase 1

- [ ] **Variété ennemis** : Tanks + Boss (définis dans enemies.toml, pas encore spawnés)
- [ ] **Sélection + clic droit** : sélection d'unités/bâtiments, ordres déplacement
- [ ] **Auto-build** : workers construisent les bâtiments file d'attente
- [ ] **Auto-route belts** : placement automatique des ceintures
- [ ] **Équilibrage** : coûts, HP, timing via TOML
- [ ] **Tests** : placement, pathfinding, combat
- [ ] **Déploiement web** (itch.io / GitHub Pages)
- [ ] **Polissage final** : textures (PNG/SVG), animations, performances

---

## Phase 2 — Vers Factorio

Quand le scaffold TD est stable, la roadmap devient :

### Map & Monde

- [ ] **Chunks** : découpage en chunks 32×32, loading/unloading
- [ ] **Génération procédurale** : biomes, obstacles, ressources réparties
- [ ] **Carte infinie** : extension dynamique à la demande
- [ ] **Pathfinding hiérarchique** : A* inter-chunks + BFS intra-chunk

### Économie & Craft

- [ ] **N ressources** : passage à des IDs dynamiques (String)
- [ ] **Recettes arborescentes** : inputs/outputs multiples, temps, selecteurs
- [ ] **Tech tree** : `data/techs.toml`, déblocages progressifs
- [ ] **Niveaux de bâtiments** (Miner II, Assembler III, etc.)

### Multijoueur

- [ ] **Simulation déterministe** (GameSeed, FrameNumber)
- [ ] **NetworkId** sur entités persistantes
- [ ] **Command pattern** (Events → inputs réseau)
- [ ] **P2P / serveur** : quinn, matchmaking
- [ ] **Anti-triche** : hashs, replay

### Contenu

- [ ] **Plus de bâtiments** : Reactor, Radar, Repair Tower, etc.
- [ ] **Plus de récettes** : circuits, modules, composants
- [ ] **Mode campagne** : niveaux avec objectifs
- [ ] **Mode survie infini** : vagues infinies
- [ ] **Mode bac à sable** : créatif sans ennemis

### Rendu & Polissage

- [ ] **Sprites/atlas** : remplacer les Mesh2d shapes
- [ ] **Animations** : ennemis, unités, bâtiments
- [ ] **Fog of war**
- [ ] **Minimap**
- [ ] **Build optimisé** (LTO, release profile)
- [ ] **Packaging** Windows/Mac/Linux

---

## Dépendances

```
Phase 1 (scaffold TD stable)
    │
    ├── Chunks ──→ Carte infinie ──→ Pathfinding hiérarchique
    │
    ├── N ressources ──→ Craft arborescent ──→ Tech tree
    │
    ├── Sélection + ordres ──→ PvP solo ──→ Multi
    │
    └── Sprites ──→ animations ──→ polish
```
