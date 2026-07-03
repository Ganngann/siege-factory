# Game Design — Siege Factory

## Vision (destination)

Un jeu **Factorio-like** : carte infinie, multijoueur, arbre technologique profond, recettes en arborescence, N ressources.

Tout ce qui est fait aujourd'hui (tower defense, vagues, grille 20×15) est un **scaffold temporaire** pour construire le socle technique en itérant sur un gameplay simple mais complet.

Le mode TD actuel restera comme un mode de jeu (défense de base) une fois la destination atteinte.

---

## Phase actuelle — Scaffold TD

### Flow joueur

1. **Début de partie** : carte 20×15 avec gisements fixes. Le joueur place son quartier général.
2. **Phase automation** : le joueur place mines, assembleurs, ceintures pour produire ressources et munitions.
3. **Phase défense** : des vagues d'ennemis spawnent et marchent vers la base via pathfinding BFS.
4. **Phase RTS** : sélection d'unités, ordres de déplacement, attaque ciblée.
5. **Win/Loss** : survivre à 10 vagues, ou base détruite = game over.

### Ressources actuelles

Définies dans `data/resources.toml`.

| Ressource | Source | Usage |
|---|---|---|
| Ore | Mines (automatique) | Construction, ammo |
| Ammo | Assembleur (Ore → Ammo) | Tourelles |
| Energy | Réacteurs (plus tard) | Alimentation buildings (plus tard) |

Principes :
- Les ressources sont transportées par ceintures et stockées dans des inventaires.
- Toute production est automatique une fois les buildings placés.

### Bâtiments actuels

Définis dans `data/buildings.toml`.

| Building | Rôle |
|---|---|
| HQ | Centre, HP de la base, stockage global |
| Miner | Extrait Ore des gisements |
| Assembler | Transforme Ore → Ammo |
| Belt | Transporte les items entre buildings |
| Wall | Bloque les ennemis, HP élevé |
| Turret | Tire automatiquement sur ennemis |
| Storage | Stockage tampon, capacité 64 |
| Splitter | Route les items sur 2 sorties |
| Sorter | Filtre les items par type |

### Ennemis actuels

Définis dans `data/enemies.toml`. Pathfinding : BFS sur grille 20×15.

| Type | Comportement |
|---|---|
| Runner | Rapide, faible |
| Tank | Lent, résistant |

### Win / Loss (phase TD)

- **Win** : survivre à 10 vagues (WIN_WAVES = 10).
- **Loss** : HQ détruit (HP = 0).

---

## Destination — Factorio-like

### Carte

- Infinie / à étendue dynamique (chunks 32×32)
- Génération procédurale avec seed déterministe
- Biomes, obstacles naturels, ressources réparties

### Économie & Craft

- N ressources de base
- Recettes en arborescence (ex: Ore → Plaques → Circuits → Modules → ...)
- Usine automatisée, transport par ceintures/trains/drones

### Technologie

- Arbre de recherche débloquant bâtiments, recettes, améliorations
- Niveaux de bâtiments (Miner II, Assembler III, etc.)

### Multijoueur

- Simulation déterministe (même seed, frame number)
- Mode coop (vagues + dures) et PvP (plusieurs variantes)
- NetworkId sur entités persistantes

### Anti-microgestion

Tout ce qui est répétitif doit être automatisable. Le joueur design l'usine, ne l'exploite pas manuellement.

- Production continue automatique
- Ceintures auto
- Tourelles auto avec priorité
- Ghost placement, blueprints
- Rally points, patrouilles, auto-squad

---

## Principe d'évolution

Chaque feature implémentée dans le scaffold TD est un **investissement** qui servira dans la destination :

| Scaffold TD | Sert la destination |
|---|---|
| ECS + Events | Scale multi, determinism |
| Data-driven TOML | N ressources, modding |
| Menu arborescent | Tech tree, recettes |
| BFS pathfinding | Sera remplacé par pathfinding hiérarchique (chunk A* + BFS local) |
| Inventory component | Reste inchangé |
| Ceintures + items | Core du transport, reste inchangé |
