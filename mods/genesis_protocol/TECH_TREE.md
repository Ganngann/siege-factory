# Arbre Technologique

## Vue d'ensemble

| Phase | Titre | Durée | État capsule | Mécanique clé | Réparation capsule |
|-------|-------|-------|-------------|--------------|-------------------|
| 0 | Réveil | 2-5h | Éteinte, vitre noire | Craft manuel, outils pierre | Dégager les débris, ouvrir les panneaux d'accès |
| 1 | Étincelle | 10-15h | Voyants d'alimentation verts | Énergie charbon, minerais, fours | Restaurer l'alimentation électrique primaire |
| 2 | Rouille et Vapeur | 15-20h | Liquide visible, bulles | Fluides, vapeur, acier, belts | Rebrancher le circuit de refroidissement |
| 3 | Fil du Cuivre | 20-30h | Lueur interne faible | Électricité, circuits, chimie | Remplacer le noyau informatique calciné |
| 4 | Pouls | 30-40h | Battement cardiaque | Moteurs, batteries, logistique | Débloquer les pompes internes et actionneurs |
| 5 | Nanites | 40-50h | Vitre partiellement transparente | Nano-assemblage, forage profond | Souder les microfissures de la coque interne |
| 6 | Genèse | 40-50h | Signes vitaux stables | Bio-ingénierie, tissus | Synthétiser les fluides biologiques du réservoir de stase |
| Final | Premier Souffle | ~20h | Vitre claire, main | Assemblage humain | Amorcer la séquence de réveil |

Chaque phase se débloque par une **découverte** : le joueur craft un objet clé, ce qui déclenche un log crypté et débloque le palier suivant de la capsule + les nouveaux bâtiments/recettes associés.

---

## Phase 0 — Réveil (2-5h)

*Capsule : éteinte, aucun signe de vie.*
*Réparation : dégager les débris, ouvrir les panneaux d'accès.*

Tu émerges de la capsule. La ville est en ruine. Tu n'as que tes mains.

**Outils** (craft main, usage permanent) :
- stone_axe → coupe les arbres
- stone_pickaxe → casse la pierre et le charbon de surface
- stone_blade → coupe les herbes (plant_fiber)
- hammer → assemble les bâtiments au workbench

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
**Objectif pour débloquer la phase suivante** : Crafter stone_pickaxe + hammer → dégager les débris autour de la capsule et ouvrir les panneaux d'accès → l'écran de la capsule affiche *"ALIMENTATION REQUISE"*

---

## Phase 1 — Étincelle (10-15h)

*Capsule : voyants d'alimentation verts. Un frémissement.*
*Réparation : restaurer l'alimentation électrique primaire.*

Le charbon change tout. La combustion devient moteur.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Burner Generator | 4 iron_parts + 2 stone | Charbon → energy |
| Manual Miner | 3 iron_parts + 1 gear | Mine un minerai automatiquement |
| Furnace | 4 stone_brick + 2 ceramic + 2 iron_parts | Mineral + charbon → lingot |
| Anvil | 4 iron_parts + 2 stone | Outils et composants métal |

**Nouvelles ressources** : iron_ore, copper_ore, coal, iron_ingot, copper_ingot, energy

**Recettes** :
- Furnace : iron_ore + coal → iron_ingot ; copper_ore + coal → copper_ingot ; clay → ceramic ; sand + coal → silicon
- Anvil : copper_ingot → copper_wire

**Découverte** : Première production d'energy → *"Une étincelle dans le noir"*
**Objectif pour débloquer la phase suivante** : Construire un Burner Generator à côté de la capsule et le brancher → les voyants d'alimentation s'allument → l'écran affiche *"CIRCUIT DE REFROIDISSEMENT DÉFAILLANT"*

---

## Phase 2 — Rouille et Vapeur (15-20h)

*Capsule : liquide visible, condensation sur la vitre.*
*Réparation : rebrancher le circuit de refroidissement.*

La vapeur est la clé. Belts. Acier. Transport automatisé.

**Mécaniques** : Fluides (eau, vapeur). Belts (transport d'items).

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Water Pump | 3 iron_parts + 1 pipe | Pompe l'eau (nappe) |
| Steam Generator | 4 iron_parts + 2 copper + 2 pipe | Eau + chaleur → steam |
| Blast Furnace | 6 steel + 4 stone_brick | Fer → acier (vapeur) |
| Gear Press | 3 iron_parts + 2 rope + anvil | Lingot → gear / screw |
| Pipe | 2 iron_parts → 2 pipe | Transport fluides |
| Belt | 2 iron_parts + 1 gear + 2 rope | Transport items |
| Splitter | 2 iron + 2 gear | Divise/combine flux belt |

**Nouvelles ressources** : water, steam, steel, gear, screw, pipe

**Recettes** :
- Gear Press : iron_ingot → gear ; copper_ingot → screw
- Blast Furnace : iron_ingot + coal + steam → steel
- Steam Generator : water + coal → steam + energy
- Water Pump : eau de nappe → water
- Anvil : iron_ingot → pipe
- Assemblage : iron_parts + gear → belt ; iron_ingot + gear → splitter

**Découverte** : Première vapeur produite → *"La machine s'éveille"*
**Objectif pour débloquer la phase suivante** : Fabriquer Water Pump + tuyaux + Steam Generator → raccorder le circuit de refroidissement de la capsule → condensation sur la vitre → l'écran affiche *"CPU CALCINÉ"*

---

## Phase 3 — Fil du Cuivre (20-30h)

*Capsule : lueur interne faible. Le cœur bat faiblement.*
*Réparation : remplacer le noyau informatique calciné.*

L'électricité. Les circuits. La chimie.

**Mécaniques** : Électricité (power grid). Circuits. Chimie (pétrole).

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Electric Generator | 4 steel + 2 copper_wire + 4 screw + 2 gear | Charbon → électricité |
| Power Pole | 2 iron + 1 copper_wire | Grid électrique |
| Assembler | 4 iron + 2 circuit + 1 motor | Craft automatisé |
| Chemical Lab | 4 steel + 2 glass + 2 pipe | Pétrole → plastique + chimie |
| Oil Pump | 4 steel + 2 gear + 1 motor | Pétrole brut (gisement) |
| Storage Chest | 4 iron + 2 planks | Stockage étendu |

**Nouvelles ressources** : copper_wire, circuit, petroleum_gas, plastic, glass

**Recettes** :
- Assembler : copper → copper_wire ; wire + iron → circuit ;
  scrap_metal → iron_parts ; wood → planks ; stone → stone_brick ; plant_fiber → rope
- Chemical Lab : crude_oil → petroleum_gas ; gas + sulfur → plastic ; sulfur + water → acid
- Furnace : sand + coal → glass

**Découverte** : Premier circuit → *"Un cerveau de cuivre"*
**Objectif pour débloquer la phase suivante** : Crafter advanced_circuit → l'insérer dans le slot CPU de la capsule → le noyau informatique reboote → l'écran affiche *"POMPES INTERNES BLOQUÉES"*

---

## Phase 4 — Pouls (30-40h)

*Capsule : battement cardiaque visible, signes vitaux faibles.*
*Réparation : débloquer les pompes internes et actionneurs.*

Moteurs. Batteries. Logistique avancée. La base devient une usine.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Motor Foundry | 6 steel + 4 circuit + 4 screw + 2 gear | steel + wire + copper → motor |
| Battery Station | 4 steel + 2 plastic + 2 copper | acid + metal → battery |
| Electronics Lab | 6 steel + 4 circuit + 2 glass | circuit → advanced_circuit ; electronic_module |
| Assembly Crane | 8 steel + 4 motor + 2 circuit + 4 screw | motor + steel → machine_frame |
| Aerial Belt | 4 steel + 2 motor + 2 gear | Belt aérien (par-dessus) |
| Sorter | 4 iron + 2 circuit + 1 motor | Trie les items sur belt |

**Nouvelles ressources** : motor, battery, advanced_circuit, electronic_module, machine_frame, concrete, silicon

**Recettes** :
- Motor Foundry : steel + copper_wire + copper_ingot → motor
- Battery Station : acid + copper_ingot + steel → battery
- Electronics Lab : circuit + silicon → advanced_circuit ; advanced_circuit + wire → electronic_module
- Assembly Crane : motor + steel + circuit → machine_frame
- Forge : steel + motor + gear → aerial_belt ; iron_ingot + circuit + motor → sorter

**Découverte** : Premier moteur → *"Le pouls de la machine"*
**Objectif pour débloquer la phase suivante** : Crafter motor + battery → les installer dans les actionneurs de la capsule → les pompes internes se débloquent → l'écran affiche *"MICROFISSURES COQUE DÉTECTÉES"*

---

## Phase 5 — Nanites (40-50h)

*Capsule : vitre partiellement transparente, silhouette visible.*
*Réparation : souder les microfissures de la coque interne.*

Construction atomique. Ressources infinies en profondeur.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Nanite Assembler | 8 steel + 4 processor + 2 laser + 2 electronic_module | Craft nano-scale |
| Deep Core Drill | 10 steel + 4 motor + 2 machine_frame + 2 electronic_module | Minerai infini |
| Compactor | 6 steel + 2 motor + 2 gear + 1 battery | Compresse 4:1 |
| High-Speed Belt | 6 steel + 2 motor + 2 circuit | Belt 3x plus rapide |
| Excavation Rig | 8 steel + 4 motor + 2 machine_frame | Terrain massif |

**Nouvelles ressources** : processor, laser_crystal, nano_pack, composite, alloy

**Recettes** :
- Nanite Assembler : advanced_circuit + silicon → processor ; processor + energy → nano_pack
- Chemical Lab (avancé) : steel + plastic + silicon → composite ; steel + copper + silicon → alloy ; glass + acid + energy → laser_crystal

**Découverte** : Premier nano_pack → *"L'infiniment petit"*
**Objectif pour débloquer la phase suivante** : Crafter nano_pack → activer les nanites sur la coque de la capsule → les microfissures sont scellées → l'écran affiche *"FLUIDES BIOLOGIQUES MANQUANTS"*

---

## Phase 6 — Genèse (40-50h)

*Capsule : signes vitaux stables, main visible contre la vitre.*
*Réparation : synthétiser les fluides biologiques du réservoir de stase.*

Construire un corps. Bio-ingénierie.

**Bâtiments** :
| Bâtiment | Craft | Fonction |
|----------|-------|----------|
| Bio-Lab | 6 steel + 4 processor + 2 glass | Composés organiques, enzymes |
| Tissue Cultivator | 8 steel + 4 nano_pack + 2 laser | Culture cellulaire → stem_cells |
| Synthesizer | 6 steel + 4 motor + 2 pipe | Synthèse protéines |
| Scanner Array | 8 steel + 4 processor + 2 radar + 4 electronic_module | Cartographie génétique → neural_map |
| Bio-Printer | 10 steel + 4 nano_pack + 4 processor + 2 composite | Impression 3D organique |

**Nouvelles ressources** : organic_compound, enzyme, protein, synthetic_blood, neural_map, stem_cells, bio_mass

**Recettes** :
- Bio-Lab : plant_fiber + water → organic_compound ; organic_compound + protein → enzyme
- Tissue Cultivator : organic_compound + enzyme + energy → stem_cells
- Synthesizer : organic_compound + energy → protein ; protein + iron_ingot + energy → synthetic_blood
- Scanner Array : advanced_circuit + processor + energy → neural_map
- Bio-Printer : stem_cells + protein + nano_pack → bio_mass

**Découverte** : Premier composé organique → *"La vie jaillit du métal"*
**Objectif pour débloquer la phase suivante** : Synthétiser organic_compound + enzyme + protein + synthetic_blood + stem_cells → les injecter dans le réservoir de stase → les signes vitaux apparaissent → l'écran affiche *"SÉQUENCE DE RÉVEIL PRÊTE"*

---

## Phase Finale — Premier Souffle (~20h)

*Capsule : tous les systèmes au vert. Vitre claire.*
*Réparation : amorcer la séquence de réveil.*

La capsule accepte les derniers composants.

**Les 4 composants ultimes** :

| Composant | Craft | Bâtiment |
|-----------|-------|----------|
| Neural Interface | processor + neural_map | Assembler |
| Synthetic Heart | motor + synthetic_blood + protein + bio_mass | Assembly Crane |
| Genome Sequence | stem_cells + neural_map + nano_pack + bio_mass | Bio-Lab |
| Fusion Core | alloy + composite + laser_crystal + nano_pack | Nanite Assembler |

**Rituel** : Chaque composant est inséré dans la Genesis Array (capsule).
Les 4 insérés → compte à rebours 60s → la capsule s'illumine.

**Objectif final** : Assembler les 4 composants ultimes → les insérer dans la capsule → la capsule s'illumine → main contre la vitre → *premier souffle*

**Fin** : Une main nue se pose contre la vitre. Un premier souffle.

---

## Récapitulatif — Ressources par famille

| Famille | Ressources |
|---------|-----------|
| **Minerais bruts** | scrap_metal, iron_ore, copper_ore, coal, stone, sand, clay, sulfur, crude_oil |
| **Métaux** | iron_parts, iron_ingot, copper_ingot, steel, alloy |
| **Minéraux transformés** | stone_brick, ceramic, glass, concrete |
| **Composants** | gear, screw, copper_wire, pipe, circuit, motor, battery, electronic_module, machine_frame |
| **Chimie** | petroleum_gas, plastic, acid, silicon |
| **Avancé** | advanced_circuit, processor, laser_crystal, composite, nano_pack, fusion_core |
| **Biologique** | organic_compound, enzyme, protein, synthetic_blood, neural_map, stem_cells, bio_mass |
| **Organique brut** | wood, plant_fiber, planks, rope, rubber |
| **Fluides** | water, steam |
| **Énergie** | energy |

## Récapitulatif — Bâtiments par phase

| Phase | Bâtiments |
|-------|-----------|
| 0 | Workbench, Campfire |
| 1 | Burner Generator, Manual Miner, Furnace, Anvil |
| 2 | Water Pump, Steam Generator, Blast Furnace, Gear Press, Belt, Splitter, Pipe |
| 3 | Electric Generator, Power Pole, Assembler, Chemical Lab, Oil Pump, Storage Chest |
| 4 | Motor Foundry, Battery Station, Electronics Lab, Assembly Crane, Aerial Belt, Sorter |
| 5 | Nanite Assembler, Deep Core Drill, Compactor, High-Speed Belt, Excavation Rig |
| 6 | Bio-Lab, Tissue Cultivator, Synthesizer, Scanner Array, Bio-Printer |
| Final | Genesis Array |
