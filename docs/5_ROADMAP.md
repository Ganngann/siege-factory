# Roadmap — Siege Factory

## Légende

- **M** = Milestone
- Durée estimée en sessions de dev (1 session = ~2-4h)

## M1 — Squelette ✅ (livré)

- [x] Projet Bevy, structure des dossiers
- [x] Grille tuilée 20×15, rendu damier
- [x] Machine à états : Loading → Playing → GameOver
- [x] Clic sur tuile → highlight
- [x] Tests unitaires (13 tests) + proptest

## M2 — Économie (2-3 sessions)

- [ ] Système data-driven : registres chargés depuis `data/*.toml`
- [ ] ResourceId + ResourceRegistry
- [ ] Inventory component
- [ ] Gisements Ore placés sur la carte
- [ ] Construction HQ + Miner cliquable
- [ ] Production : Miner → Ore automatique
- [ ] Tests : production, inventaire, registres

## M3 — Constructions (2-3 sessions)

- [ ] BuildingRegistry + BuildingDef
- [ ] Palette de construction UI (choisir building)
- [ ] Placement système (clic → building apparaît si terrain libre + coût payé)
- [ ] Assembler : Ore → Ammo
- [ ] Ceinture transporteuse : items entre buildings
- [ ] Turret : tire sur ennemis dans son range
- [ ] Tests : placement, coûts, chaîne de production

## M4 — Ennemis & combat (3-4 sessions)

- [ ] EnemyRegistry + EnemyDef
- [ ] WaveSpawner : timer, composition, difficulté croissante
- [ ] A* pathfinding sur grille carrée
- [ ] Ennemis marchent vers HQ, contournent les murs
- [ ] Tourelles tirent, projectiles, dégâts
- [ ] HQ HP, game over si détruit
- [ ] Win si toutes les vagues survivent
- [ ] Tests : pathfinding, spawn, combat, game loop

## M5 — Polissage & web (2 sessions)

- [ ] Menu principal
- [ ] Écran de fin (score, vague atteinte)
- [ ] Build WASM (ciblage navigateur)
- [ ] Playtest en ligne
- [ ] Ajustements équilibrage

## M6 — Évolution RTS (3-4 sessions)

- [ ] Unités mobiles (ouvriers, soldats)
- [ ] Sélection box (drag-select)
- [ ] Ordres : déplacement, attaque, récolte
- [ ] Arbre de technologies
- [ ] Fog of war
- [ ] Mode survival infini

## M7 — Multijoueur (4-6 sessions)

- [ ] Simulation déterministe (GameSeed, FrameNumber)
- [ ] NetworkId sur entités clés
- [ ] Command pattern (Events → inputs réseau)
- [ ] Connexion P2P (quinn)
- [ ] Synchronisation d'état
- [ ] Mode PvP (écran partagé d'abord)
- [ ] Serveur notaire (matchmaking, ELO)
- [ ] Anti-triche (hashs, replay)

## M8 — Release Steam (2 sessions)

- [ ] Integration Steamworks (steamworks-rs)
- [ ] Build optimisé (LTO, profile release)
- [ ] Tuning perf, profiling
- [ ] Packaging Windows/Mac/Linux

## Dépendances

```
M1 → M2 → M3 → M4 → M5
  ↘              ↘
   M6 ──────────→ M7 → M8
```

M6 peut commencer en parallèle de M4 si besoin.
