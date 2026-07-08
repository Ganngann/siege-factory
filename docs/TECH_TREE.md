# Arbre Technologique

## Vue d'ensemble

| Phase | Titre | Durée | État capsule | Mécanique clé |
|-------|-------|-------|-------------|--------------|
| 0 | Réveil | 2-5h | Éteinte, vitre noire | Craft manuel, outils pierre |
| 1 | Étincelle | 10-15h | Voyants d'alimentation verts | Énergie charbon, minerais, fours |
| 2 | Rouille et Vapeur | 15-20h | Liquide visible, bulles | Fluides, vapeur, acier, belts |
| 3 | Fil du Cuivre | 20-30h | Lueur interne faible | Électricité, circuits, chimie |
| 4 | Pouls | 30-40h | Battement cardiaque | Moteurs, batteries, logistique |
| 5 | Nanites | 40-50h | Vitre partiellement transparente | Nano-assemblage, forage profond |
| 6 | Genèse | 40-50h | Signes vitaux stables | Bio-ingénierie, tissus |
| Final | Premier Souffle | ~20h | Vitre claire, main | Assemblage humain |

Chaque phase se débloque par une **découverte** : le joueur craft un objet clé, ce qui déclenche un log crypté et débloque le palier suivant de la capsule + les nouveaux bâtiments/recettes associés.

---

## Phase 0 — Réveil (2-5h)

*Capsule : éteinte, aucun signe de vie.*

Tu émerges de la capsule. La ville est en ruine. Tu n'as que tes mains.

**Outils** (craft main) : stone_axe, stone_pickaxe, hammer, stone_blade

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Workbench | 2 wood + 4 stone | Station de craft manuel |
| Campfire | 3 stone + 2 wood | Cuisson, fusion basique |

**Ressources** : scrap_metal (ruines), wood (arbres), stone (sol), clay (sol humide), plant_fiber (herbes)

**Recettes** :
- Workbench : scrap_metal → iron_parts ; wood → planks ; stone → stone_brick ; clay → ceramic ; plant_fiber → rope
- Campfire : clay → ceramic (x2) ; stone → stone_brick (x2) ; iron_parts + coal → iron_ingot

**Découverte** : Premier usage du Campfire → *"Le feu brûle encore"*

---

## Phase 1 — Étincelle (10-15h)

*Capsule : voyants d'alimentation verts. Un frémissement.*

Le charbon change tout. La combustion devient moteur.

**Outils** : iron_pickaxe, iron_axe

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Burner Generator | 4 iron_parts + 2 stone | Charbon → energy |
| Manual Miner | 3 iron_parts + 1 gear | Mine un minerai automatiquement |
| Furnace | 6 stone_brick + 2 iron_parts | Mineral + charbon → lingot |
| Anvil | 4 iron_parts + 2 stone | Outils et composants métal |

**Nouvelles ressources** : iron_ore, copper_ore, coal, iron_ingot, copper_ingot, energy

**Recettes** :
- Furnace : iron_ore + coal → iron_ingot ; copper_ore + coal → copper_ingot
- Anvil : iron_ingot → iron_pickaxe, iron_axe ; copper_ingot → copper_wire

**Découverte** : Première production d'energy → *"Une étincelle dans le noir"*

---

## Phase 2 — Rouille et Vapeur (15-20h)

*Capsule : liquide visible, condensation sur la vitre.*

La vapeur est la clé. Belts. Acier. Transport automatisé.

**Mécaniques** : Fluides (eau, vapeur). Belts (transport d'items).

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Water Pump | 3 iron_parts + 1 pipe | Pompe l'eau (nappe) |
| Steam Generator | 4 iron_parts + 2 copper + 2 pipe | Eau + chaleur → steam |
| Blast Furnace | 6 steel + 4 stone_brick | Fer → acier (vapeur) |
| Gear Press | 3 iron_parts + anvil | Lingot → gear / screw |
| Pipe | 2 iron_parts → 2 pipe | Transport fluides |
| Belt | 2 iron_parts + 1 gear | Transport items |
| Underground Belt | 4 iron + 2 gear | Passe sous les bâtiments |
| Splitter | 2 iron + 2 gear | Divise/combine flux belt |

**Nouvelles ressources** : water, steam, steel, gear, screw, pipe

**Découverte** : Première vapeur produite → *"La machine s'éveille"*

---

## Phase 3 — Fil du Cuivre (20-30h)

*Capsule : lueur interne faible. Le cœur bat faiblement.*

L'électricité. Les circuits. La chimie. La défense.

**Mécaniques** : Électricité (power grid). Circuits. Chimie (pétrole). Tourelles.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Electric Generator | 4 steel + 2 copper_wire + 2 gear | Charbon → électricité |
| Power Pole | 2 iron + 1 copper_wire | Grid électrique |
| Assembler | 4 iron + 2 circuit + 1 motor | Craft automatisé |
| Chemical Lab | 4 steel + 2 glass + 2 pipe | Pétrole → plastique + chimie |
| Oil Pump | 4 steel + 2 gear + 1 motor | Pétrole brut (gisement) |
| Turret | 4 iron + 2 gear + 1 circuit | Défense automatique |
| Storage Chest | 4 iron + 2 planks | Stockage étendu |

**Nouvelles ressources** : copper_wire, circuit, petroleum_gas, plastic, sulfur_powder, glass, ammo

**Recettes** :
- Assembler : copper → copper_wire ; wire + iron → circuit ; iron + copper → ammo
- Chemical Lab : crude_oil → petroleum_gas ; gas → plastic ; sulfur → sulfur_powder

**Découverte** : Premier circuit → *"Un cerveau de cuivre"*

---

## Phase 4 — Pouls (30-40h)

*Capsule : battement cardiaque visible, signes vitaux faibles.*

Moteurs. Batteries. Logistique avancée. La base devient une usine.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Motor Foundry | 6 steel + 4 circuit + 2 gear | steel + wire + copper → motor |
| Battery Station | 4 steel + 2 plastic + 2 copper | acid + metal → battery |
| Electronics Lab | 6 steel + 4 circuit + 2 glass | circuit → advanced_circuit ; electronic_module |
| Assembly Crane | 8 steel + 4 motor + 2 circuit | motor + steel → machine_frame |
| Aerial Belt | 4 steel + 2 motor + 2 gear | Belt aérien (par-dessus) |
| Sorter | 4 iron + 2 circuit + 1 motor | Trie les items sur belt |
| Wall Mk2 | 6 steel + 2 concrete | Mur renforcé |

**Nouvelles ressources** : motor, battery, advanced_circuit, electronic_module, machine_frame, concrete

**Découverte** : Premier moteur → *"Le pouls de la machine"*

---

## Phase 5 — Nanites (40-50h)

*Capsule : vitre partiellement transparente, silhouette visible.*

Construction atomique. Ressources infinies en profondeur.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Nanite Assembler | 8 steel + 4 processor + 2 laser | Craft nano-scale |
| Deep Core Drill | 10 steel + 4 motor + 2 machine_frame | Minerai infini |
| Compactor | 6 steel + 2 motor + 2 gear | Compresse 4:1 |
| Laser Turret | 6 steel + 2 laser_crystal + 2 circuit | Dégâts élevés, longue portée |
| High-Speed Belt | 6 steel + 2 motor + 2 circuit | Belt 3x plus rapide |
| Excavation Rig | 8 steel + 4 motor + 2 machine_frame | Terrain massif |

**Nouvelles ressources** : processor, laser_crystal, nano_pack, composite, alloy

**Recettes** :
- Nanite Assembler : advanced_circuit + silicon → processor ; processor + energy → nano_pack
- Chemical Lab (avancé) : steel + plastic → composite ; steel + copper → alloy

**Découverte** : Premier nano_pack → *"L'infiniment petit"*

---

## Phase 6 — Genèse (40-50h)

*Capsule : signes vitaux stables, main visible contre la vitre.*

Construire un corps. Bio-ingénierie. Drones automatisés.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Bio-Lab | 6 steel + 4 processor + 2 glass | Composés organiques, enzymes |
| Tissue Cultivator | 8 steel + 4 nano_pack + 2 laser | Culture cellulaire → stem_cells |
| Synthesizer | 6 steel + 4 motor + 2 pipe | Synthèse protéines |
| Scanner Array | 8 steel + 4 processor + 2 radar | Cartographie génétique → neural_map |
| Bio-Printer | 10 steel + 4 nano_pack + 4 processor | Impression 3D organique |
| Drone Hub | 8 steel + 4 processor + 2 motor | Drones logistiques autonomes |

**Nouvelles ressources** : organic_compound, enzyme, protein, synthetic_blood, neural_map, stem_cells

**Découverte** : Premier composé organique → *"La vie jaillit du métal"*

---

## Phase Finale — Premier Souffle (~20h)

*Capsule : tous les systèmes au vert. Vitre claire.*

La capsule accepte les derniers composants.

**Les 4 composants ultimes** :

| Composant | Craft | Bâtiment |
|-----------|-------|----------|
| Neural Interface | processor + neural_map | Assembler |
| Synthetic Heart | motor + synthetic_blood + protein | Assembly Crane |
| Genome Sequence | stem_cells + neural_map + nano_pack | Bio-Lab |
| Fusion Core | alloy + laser_crystal + nano_pack | Nanite Assembler |

**Rituel** : Chaque composant est inséré dans la Genesis Array (capsule).
Les 4 insérés → compte à rebours 60s → la capsule s'illumine.

**Fin** : Une main nue se pose contre la vitre. Un premier souffle.

---

## Récapitulatif — Ressources par famille

| Famille | Ressources |
|---------|-----------|
| **Minerais bruts** | scrap_metal, iron_ore, copper_ore, coal, stone, sand, clay, sulfur, crude_oil |
| **Métaux** | iron_ingot, copper_ingot, steel, alloy |
| **Minéraux transformés** | stone_brick, ceramic, glass, concrete |
| **Composants** | gear, screw, copper_wire, pipe, circuit, motor, battery, electronic_module, machine_frame |
| **Chimie** | petroleum_gas, plastic, sulfur_powder, lubricant, acid |
| **Avancé** | advanced_circuit, processor, laser_crystal, composite, nano_pack, fusion_core |
| **Biologique** | organic_compound, enzyme, protein, synthetic_blood, neural_map, stem_cells |
| **Organique brut** | wood, plant_fiber, rubber |
| **Fluides** | water, steam |
| **Énergie** | energy |
| **Spécial** | ammo |

## Récapitulatif — Bâtiments par phase

| Phase | Bâtiments |
|-------|-----------|
| 0 | Workbench, Campfire |
| 1 | Burner Generator, Manual Miner, Furnace, Anvil |
| 2 | Water Pump, Steam Generator, Blast Furnace, Gear Press, Belt, Underground Belt, Splitter, Pipe |
| 3 | Electric Generator, Power Pole, Assembler, Chemical Lab, Oil Pump, Turret, Storage Chest |
| 4 | Motor Foundry, Battery Station, Electronics Lab, Assembly Crane, Aerial Belt, Sorter, Wall Mk2 |
| 5 | Nanite Assembler, Deep Core Drill, Compactor, Laser Turret, High-Speed Belt, Excavation Rig |
| 6 | Bio-Lab, Tissue Cultivator, Synthesizer, Scanner Array, Bio-Printer, Drone Hub |
| Final | Genesis Array |
