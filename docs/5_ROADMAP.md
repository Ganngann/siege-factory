# Roadmap — Siege Factory

## Légende

- [x] = fait (stable)
- [-] = partiel / à corriger
- [ ] = à faire
- Durée estimée en sessions de dev (1 session = ~2-4h)

---

## Phase 2 — Contenu étendu

### Économie & Production
- [x] **Ressources cultivables** (note #10) : agriculture, fermes, cultivateurs, cultures (blé/bois), récolte, livraison, spirale
- [x] **Système Découverte + Archive** : `data/discoveries.toml`, compteur de crafts par bâtiment, découvertes fragiles, Archive pérennise
- [x] **Recettes arborescentes** : 15 ressources sur 6 niveaux (Ore → Plate → Gear → Motor → Drivetrain, etc.)
- [x] **Système d'électricité** (note #14) : réseau électrique, poteaux, générateurs, consommateurs, section POWER inspect
- [ ] **Belts souterrains / aériens** (note #15) : variantes de belts passant sous/par-dessus les autres bâtiments
- [ ] **Niveaux de bâtiments** (Miner II, Assembler III, etc.)

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
| 12 | Arbre de technologies (→ remplacé par Découverte+Archive) | 2 ✅ |
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
