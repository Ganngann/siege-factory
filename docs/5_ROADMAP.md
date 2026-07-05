# Roadmap — Siege Factory

## Légende

- [x] = fait (stable)
- [-] = partiel / à corriger
- [ ] = à faire
- Durée estimée en sessions de dev (1 session = ~2-4h)

---

## Phase 1A — Correctifs & Polish (immédiat)

### Production (note #2)
- [-] **Refactor production** : la barre ne doit pas avancer si les ressources ne sont pas disponibles, pas d'accumulation de crédit de temps, un seul cycle de production à la fois
- [-] **Barre progression** : ne démarre que quand tout est prêt, pas de dépassement (650/3)

### UI & Boutons
- [ ] **Bouton ON/OFF** (note #5) : afficher "OFF" quand le bâtiment est inactif
- [ ] **Police manquante** (note #6) : remplacer les caractères non rendus (carrés vides) — ×, ✕, etc. — par une police complète ou des caractères ASCII
- [ ] **Auto-select 1er élément** (note #3) : au clic sur une catégorie du menu, le premier bâtiment est automatiquement sélectionné
- [ ] **Mise en avant recettes** (note #7) : dans un bâtiment, les recettes pour lesquelles les ingrédients sont disponibles doivent être mises en évidence
- [ ] **Fenêtres déplaçables** (note #21) : possibilité de glisser une fenêtre ouverte

### Dépôts
- [ ] **Clic dépôt** (note #18) : ouvrir une fenêtre affichant le type de ressource et la quantité restante (comme un bâtiment)

### Bâtiments & Monde
- [ ] **HQ indestructible** (note #13) : impossible de détruire le HQ (retirer la condition de Game Over)
- [ ] **Rotation sprite belt** (note #17) : quand on pivote un bâtiment (touche R), le sprite suit la direction
- [ ] **Clic bâtiments connectés** (note #16) : dans la fenêtre d'un bâtiment, cliquer sur les bâtiments reliés par belts pour ouvrir leur fenêtre
- [ ] **Slots de sauvegarde multiples** (note #22)

### Crash
- [ ] **Retour menu principal** (note bonus) : le jeu plante quand on quitte une partie pour revenir au menu

---

## Phase 1B — Features légères

### Vagues & Combat (rétrogradé)
- [ ] **Toggle vagues ON/OFF** (note #1) : activer ou désactiver les vagues ennemies dans une partie
- [ ] **Jeu sans fin** (note #4, #19) : retirer la victoire à 10 vagues (WIN_WAVES), le jeu ne se termine jamais. Le combat devient optionnel (mode défense ou pacifique)
- [ ] **Mode pacifique** (PeacefulMode) : créatif sans ennemis

### Map
- [ ] **Décors sur la map** (note #9, #11) : ajouter des éléments décoratifs (arbres, rochers, etc.)

### Menu & Sprites
- [ ] **Sprites bâtiments dans le menu** (note #8) : utiliser les dessins existants dans le menu de construction (remplacer les Mesh2d shapes par les sprites)
- [ ] **Sprites dédiés par dépôt** (note #20) : chaque type de dépôt de ressource a son propre sprite

---

## Phase 2 — Contenu étendu

### Économie & Production
- [ ] **Ressources cultivables** (note #10) : agriculture, fermes, plantations
- [ ] **Arbre de technologies** (note #12) : `data/techs.toml`, déblocages progressifs de recettes et QoL
- [ ] **Système d'électricité** (note #14) : réseau électrique, générateurs, consommateurs, coupures
- [ ] **Belts souterrains / aériens** (note #15) : variantes de belts passant sous/par-dessus les autres bâtiments
- [ ] **Niveaux de bâtiments** (Miner II, Assembler III, etc.)
- [ ] **Recettes arborescentes** : inputs/outputs multiples, temps, sélecteurs

### Map & Monde
- [ ] **Chunks** : découpage en chunks 32×32, loading/unloading
- [ ] **Génération procédurale** : biomes, obstacles, ressources réparties
- [ ] **Carte infinie** : extension dynamique à la demande
- [ ] **Pathfinding hiérarchique** : A* inter-chunks + BFS intra-chunk
- [ ] **Fog of war**

### Multijoueur (futur lointain)
- [ ] **Simulation déterministe** (GameSeed, FrameNumber)
- [ ] **NetworkId** sur entités persistantes
- [ ] **Command pattern** (Events → inputs réseau)
- [ ] **P2P / serveur** : quinn, matchmaking
- [ ] **Mode affrontement** (PvP)
- [ ] **Anti-triche** : hashs, replay

### Rendu & Polissage
- [ ] **Sprites/atlas** : remplacer les Mesh2d shapes restants
- [ ] **Animations** : ennemis, unités, bâtiments
- [ ] **Minimap**
- [ ] **Build optimisé** (LTO, release profile)
- [ ] **Packaging** Windows/Mac/Linux
- [ ] **Déploiement web** (itch.io / GitHub Pages)

---

## Dépendances

```
Phase 1A (correctifs immédiats)
    │
    ├──→ Phase 1B (features légères)
    │         │
    │         ├──→ Phase 2 (contenu étendu)
    │         │
    │         └──→ Combat optionnel / Mode pacifique
    │
    ├── Sprites bâtiments menu ──→ Sprites/atlas complet
    │
    └── Sauvegarde slots multiples ──→ Save/load robuste
```

## Notes utilisateur

Références croisées avec `docs/00_note.md` :

| Note | Sujet | Phase |
|---|---|---|
| 1 | Toggle vagues ON/OFF | 1B |
| 2 | Refactor production & barre | 1A |
| 3 | Auto-select 1er élément menu | 1A |
| 4 | Combat secondaire | 1B (rétrogradé) |
| 5 | Bouton ON/OFF affiche OFF | 1A |
| 6 | Police non rendue (carrés) | 1A |
| 7 | Mise en avant recettes dispo | 1A |
| 8 | Sprites bâtiments dans menu | 1B |
| 9 | Décors map | 1B |
| 10 | Ressources cultivables | 2 |
| 11 | + de décors (doublon #9) | 1B |
| 12 | Arbre de technologies | 2 |
| 13 | HQ indestructible | 1A |
| 14 | Électricité | 2 |
| 15 | Belts souterrain/aérien | 2 |
| 16 | Clic bâtiments connectés | 1A |
| 17 | Rotation sprite belt | 1A |
| 18 | Fenêtre dépôt au clic | 1A |
| 19 | Jeu sans fin | 1B |
| 20 | Sprites dédiés dépôts | 1B |
| 21 | Fenêtres déplaçables | 1A |
| 22 | Slots sauvegarde multiples | 1A |
| — | Crash retour menu principal | 1A |
