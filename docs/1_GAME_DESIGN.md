# Game Design — Siege Factory

## Concept

Mélange de RTS (top-down 2D), automation (Factorio-like), et tower defense. Le joueur construit une usine de production automatisée tout en repoussant des vagues d'ennemis, avec une évolution vers le PvP.

## Flow joueur

1. **Début de partie** : carte générée avec des gisements de ressources. Le joueur place son quartier général.
2. **Phase automation** : le joueur place des mines, assembleurs, ceintures pour produire des ressources et munitions.
3. **Phase défense** : des vagues d'ennemis spawnent et marchent vers la base via pathfinding A*.
4. **Phase RTS** : sélection d'unités, ordres de déplacement, attaque ciblée.
5. **Win/Loss** : survivre à toutes les vagues, ou base détruite = game over.

## Ressources

Définies dans `data/resources.toml`.

| Ressource | Source | Usage |
|---|---|---|
| Ore | Mines (automatique) | Construction, ammo |
| Ammo | Assembleur (Ore → Ammo) | Tourelles |
| Energy | Réacteurs | Alimentation buildings (plus tard) |

Principes :
- Les ressources sont transportées par ceintures et stockées dans des inventaires.
- Toute production est automatique une fois les buildings placés.

## Buildings

Définis dans `data/buildings.toml`.

| Building | Rôle | Taille |
|---|---|---|
| HQ | Centre, HP de la base, stockage global | 2×2 |
| Miner | Extrait Ore des gisements | 1×1 |
| Assembler | Transforme Ore → Ammo | 1×1 |
| Belt | Transporte les items entre buildings | 1×1 |
| Turret | Tire automatiquement sur ennemis | 1×1 |
| Wall | Bloque les ennemis, HP élevé | 1×1 |

Extension (post-MVP) :
- Reactor : produit Energy
- Radar : révèle une zone de la carte
- Repair Tower : répare les buildings proches
- Shield Generator : bouclier temporaire base

## Ennemis

Définis dans `data/enemies.toml`.

| Type | Comportement | Stats |
|---|---|---|
| Runner | Rapide, faible | Vitesse élevée, 1 HP |
| Tank | Lent, résistant | Vitesse lente, HP haut |
| Flier | Ignore les murs | Vitesse moyenne, HP bas (plus tard) |
| Boss | Très résistant, apparaît en fin de vague | HP très haut, dégâts élevés |

Pathfinding : A* sur grille carrée. Les ennemis contournent les buildings et murs.

## Vagues

Définies dans `data/waves.toml`.

- Chaque vague a un nombre et type d'ennemis.
- Intervalle fixe entre vagues (ex: 60 secondes).
- Difficulté croissante : plus d'ennemis, plus résistants, mix de types.
- Boss tous les 5 ou 10 vagues.
- Mode survival : vagues infinies avec difficulté progressive.

## Win / Loss

- **Win** : survivre à toutes les vagues (mode campagne) ou éliminer tous les ennemis.
- **Loss** : HQ détruit (HP = 0).
- En PvP (plus tard) : détruire le HQ adverse ou accumulateur de ressources X temps.

## Économie (boucle principale)

```
Miner ──ore──► Belt ──ore──► Assembler ──ammo──► Belt ──ammo──► Turret
                                                └──ammo──► HQ (stock)
```

Chaque building a un inventaire local (entrée/sortie). Les ceintures transferent automatiquement. Le joueur place les buildings et les ceintures, ensuite le flux est automatique.

## Difficulté progressive

- Vagues 1-5 : Runners uniquement
- Vagues 6-10 : Runners + Tanks
- Vagues 11-15 : Mix + premier Boss
- Vagues 16+ : Tous types, nombre croissant
