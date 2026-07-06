# Game Design — Siege Factory

## Vision

Un jeu **Factorio-like** : carte infinie, multijoueur, arbre technologique profond, recettes en arborescence, N ressources.

Le mode tower defense actuel est un **scaffold temporaire** pour construire le socle technique sur un gameplay simple mais complet. Il restera comme mode de jeu une fois la destination atteinte.

---

## Phase actuelle — Découverte & Archive

### Flow joueur

1. **Début de partie** : pose le HQ (20 000 ore de départ), construit mines + fours.
2. **Phase automation** : extract → forge → assemble → ceintures → stockage.
3. **Découverte** : un bâtiment productif (Furnace, Assembler, Miner, Farm) *découvre* une nouvelle recette après un certain nombre de crafts. La recette est utilisable **sur ce bâtiment uniquement**.
4. **Archivage** : tu craftes 1 exemplaire de l'item découvert et tu l'apportes à l'Archive. Il est consommé → la recette est débloquée **définitivement** sur tous les bâtiments.
5. **Approfondissement** : chaque nouvelle recette en débloque d'autres (gear → motor → drivetrain, circuit → electronic module → targeting computer).
6. **Construction** : les bâtiments eux-mêmes nécessitent des items craftés (plus que de l'ore brut). Le "mall" devient un vrai défi logistique.

### Principe Découverte + Archive

| Concept | Rôle |
|---|---|
| **Découverte** (usage-based) | Un bâtiment qui craft accumule de l'XP. À certains paliers (1, 10, 25, 50, 100...), il découvre une recette. |
| **Découverte fragile** | La recette n'existe que sur CE bâtiment. Si le bâtiment est détruit, la découverte est perdue. |
| **Archive** | Bâtiment 2×2 qu'on construit. Reçoit 1 exemplaire d'un item découvert, le consomme, débloque la recette pour toujours. |
| **Progression** | Jouer = automatiser plus pour découvrir plus, puis transporter les découvertes à l'Archive sans tout casser. |

### Ressources — arbre complet

Définies dans `data/resources.toml`. 15 ressources organisées en 4 étages (Extraction → Fonderie → Atelier → Haute Technologie).

Voir [`docs/20_RESOURCE_TREE.md`](20_RESOURCE_TREE.md) pour le diagramme Mermaid et les recettes détaillées.

Principes :
- Toute production est automatique une fois les buildings placés.
- Les ressources sont transportées par ceintures et stockées dans des inventaires.

### Bâtiments

Définis dans `data/buildings.toml`. Les coûts évoluent avec l'arbre :

| Building | Coût (exemple) | Rôle |
|---|---|---|
| HQ | — | Centre, stockage (20 000 ore au départ) |
| Miner | Iron Plate ×6, Gear ×2 | Extrait minerai des gisements |
| Furnace | Iron Plate ×8, Gear ×2 | Fait les recettes de fonderie |
| Assembler | Iron Plate ×10, Gear ×4, Circuit ×2 | Fait les recettes de craft |
| Belt | Iron Plate ×2 | Transporte les items |
| Wall | Iron Plate ×4 | Bloque les ennemis, HP élevé |
| Turret | Steel ×5, Gear ×3, Targeting Computer ×1 | Tire automatiquement |
| Storage | Iron Plate ×8, Gear ×2 | Stockage tampon, capacité 64 |
| Splitter | Iron Plate ×4, Gear ×1 | Route les items sur 2 sorties |
| Sorter | Iron Plate ×6, Gear ×2 | Filtre les items par type |
| Farm | Iron Plate ×8, Gear ×3 | Agriculture, capacité 64 |
| **Archive** | **Gear ×5, Iron Plate ×10** | **Musée des découvertes, pérennise les recettes** |

### Mécanique Découverte (détail)

Chaque bâtiment productif (Furnace, Assembler, Miner, Farm) a un compteur de crafts.
Les seuils de découverte sont définis dans `data/discoveries.toml` :

| Bâtiment | Seuil | Découverte |
|---|---|---|
| Furnace | 1 craft | Acier |
| Assembler | 10 crafts | Motor |
| Assembler | 25 crafts | Electronic Module |
| Assembler | 50 crafts | Drivetrain |
| Assembler | 75 crafts | Machine Frame |
| Assembler | 100 crafts | Targeting Computer |

Dès le seuil atteint, le bâtiment émet un événement `DiscoveryEvent` : la recette apparaît dans son sélecteur de recettes, utilisable immédiatement.

#### Péril de la découverte

Si le bâtiment est déconstruit ou détruit alors que la recette n'a pas encore été archivée, la recette est **perdue**. Le joueur doit redécouvrir (un autre bâtiment doit refaire assez de crafts).

L'Archive émet un toast quand elle reçoit un item correspondant à une découverte en attente.

### Recettes de démarrage

Certaines recettes sont débloquées dès le début (pas besoin de découverte) :
```
- mine_iron_ore, mine_copper_ore, mine_coal  (minage)
- iron_plate, copper_plate                    (fonderie)
- gear, circuit, ammo_craft                   (craft)
```

### Anti-microgestion

Tout ce qui est répétitif doit être automatisable. Le joueur design l'usine, ne l'exploite pas manuellement.

---

## Principe d'évolution

Chaque feature est un **investissement** qui servira dans la destination :

| Feature | Sert la destination |
|---|---|
| ECS + Events | Scale multi, determinism |
| Data-driven TOML | N ressources, modding |
| Découverte + Archive | Remplace le tech tree classique (pas de science packs) |
| Menu arborescent | Tech tree, recettes |
| Inventory component | Reste inchangé |
| Ceintures + items | Core du transport |
