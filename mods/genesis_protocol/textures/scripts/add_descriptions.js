const fs = require("fs");
const path = require("path");
const MANUAL_DIR = path.resolve(__dirname, "..", "svg", "manual");

const d = {};

// ========== RESSOURCES ==========

d.acid = "Acide — Ressource chimique Phase 3. Vue 3/4 : flacon en verte (#88FF44) contenant un liquide corrosif vif, contraste acide avec la rouille environnante. Récipient standard transparent, centre coloré. 64×64.";
d.advanced_circuit = "Circuit Avancé — Composant électronique Phase 4. Vue 3/4 open-frame : circuit imprimé multicouche vert (#44DD44) avec piste dorée sur le bord. Géométrie stricte, symétrie parfaite. Reflets métalliques francs. 64×64.";
d.alloy = "Alliage — Matériau composite Phase 5. Vue 3/4 : plaque métallique renforcée (#8888AA) avec fibres de carbone entrecroisées en surface. Géométrie stricte. Reflets acier brossé. 64×64.";
d.battery = "Batterie — Composant électrique Phase 4. Vue 3/4 : bloc vert (#33AA33) rectangulaire avec bornes métalliques sur le dessus. Géométrie stricte, symétrie parfaite. Reflets métalliques francs. 64×64.";
d.bio_mass = "Bio-Masse — Matériau organique Phase 6. Vue 3/4 : masse cellulaire arrondie lisse (#4DB6AC) avec noyau central et petites vésicules. Couleurs vibrantes qui détonnent avec l'industrie. 64×64.";
d.ceramic = "Céramique — Ressource cuite Phase 0. Vue 3/4 : pot en terre cuite (#C4956A) façonné, aspect rugueux artisanal. Forme organique, inspire le primitif par contraste avec les métaux polis des phases suivantes. 64×64.";
d.circuit = "Circuit — Composant électronique Phase 3. Vue 3/4 open-frame : circuit imprimé vert (#44AA44) avec puces et pistes argentées. Géométrie stricte, symétrie parfaite. Reflets métalliques. 64×64.";
d.clay = "Argile — Ressource naturelle Phase 0. Vue 3/4 : motte d'argile humide (#C4A882), forme organique irrégulière rugueuse. Aspect naturel brut par opposition aux composants usinés. 64×64.";
d.coal = "Charbon — Ressource minière Phase 1. Vue 3/4 : blocs anguleux facettés noirs (#444444) avec reflets. Forme organique asymétrique rugueuse. 64×64.";
d.composite = "Composite — Matériau renforcé Phase 5. Vue 3/4 : plaque stratifiée (#77AAAA) avec fibres croisées en quadrillage. Géométrie stricte, symétrie parfaite. 64×64.";
d.concrete = "Béton — Matériau de construction Phase 4. Vue 3/4 : bloc gris (#999999) avec granulats apparents et trous de banche. Géométrie stricte. 64×64.";
d.copper_ingot = "Lingot de Cuivre — Métal raffiné Phase 1. Vue 3/4 : barre métallique cuivrée (#CC8844) avec face supérieure brillante, rainures usinées et reflets chauds. Géométrie stricte, symétrie parfaite. 64×64.";
d.copper_ore = "Minerai de Cuivre — Ressource minière Phase 1. Vue 3/4 : roche brune (#D68A4C) irrégulière avec pépites cuivrées brillantes incrustées. Forme organique asymétrique rugueuse. 64×64.";
d.copper_wire = "Fil de Cuivre — Composant électrique Phase 2. Vue 3/4 : bobine de fil cuivré (#CC7733) enroulé sur support central avec extrémité libre qui se déroule. Géométrie stricte, reflets métalliques. 64×64.";
d.crude_oil = "Pétrole Brut — Ressource fluide Phase 3. Vue 3/4 : fût métallique noir (#222222) avec étiquette et goutte au bec. Contenant standard, centre coloré. 64×64.";
d.energy = "Énergie — Ressource électrique Phase 1. Vue 3/4 : éclair bleu vif (#3399DD) avec arcs électriques sur les côtés. Couleur vive pour simuler l'activité. 64×64.";
d.enzyme = "Enzyme — Composé biologique Phase 6. Vue 3/4 : structure protéique violette (#AB47BC) repliée, forme organique arrondie lisse. Couleur vibrante qui détonne avec l'industrie. 64×64.";
d.fusion_core = "Cœur à Fusion — Composant ultime Phase Finale. Vue 3/4 : sphère énergétique orange (#FFAA44) avec anneaux orbitaux inclinés et particules rayonnantes. Couleur vive, aspect actif. 64×64.";
d.gear = "Engrenage — Composant mécanique Phase 2. Vue 3/4 : roue dentée métallique (#887766) avec moyeu central et dents régulières. Géométrie stricte, symétrie parfaite. Reflets métalliques francs. 64×64.";
d.genome_sequence = "Séquence Génome — Composant ultime Phase Finale. Vue 3/4 : double hélice d'ADN verte (#66BB6A) avec barres de liaison. Forme organique lisse, couleur vibrante. 64×64.";
d.glass = "Verre — Ressource cuite Phase 3. Vue 3/4 : flacon transparent bleuté (#CCDDEE) avec liquide visible à l'intérieur. Contenant standard, centre coloré. 64×64.";
d.hammer = "Marteau — Outil Phase 0. Vue 3/4 : tête en pierre grise (#AA7733) liée à un manche en bois par des liens primitifs. Mélange de bois/pierre, aspect brut par opposition aux métaux polis. 64×64.";
d.iron_ingot = "Lingot de Fer — Métal raffiné Phase 1. Vue 3/4 : barre métallique grise (#AAAAAA) avec face supérieure brillante, rainures usinées et reflets métalliques francs. Géométrie stricte, symétrie parfaite. 64×64.";
d.iron_ore = "Minerai de Fer — Ressource minière Phase 1. Vue 3/4 : roche brune (#B35F33) irrégulière avec pépites métalliques brillantes incrustées. Forme organique asymétrique rugueuse. 64×64.";
d.iron_parts = "Pièces de Fer — Composants Phase 0. Vue 3/4 : assortiment de petites pièces métalliques (#999999) — engrenage, boulon, rondelle, barre et clou. Géométrie stricte, reflets métalliques. 64×64.";
d.laser_crystal = "Cristal Laser — Composant optique Phase 5. Vue 3/4 : cristal rose (#FF66DD) taillé en diamant avec facettes et éclat lumineux. Forme géométrique symétrique. Couleur vive. 64×64.";
d.machine_frame = "Châssis Mécanique — Structure Phase 4. Vue 3/4 open-frame : cadre métallique (#8888AA) en X entrecroisé avec renforts et rivets. Géométrie stricte, acier brossé. 64×64.";
d.motor = "Moteur — Composant mécanique Phase 4. Vue 3/4 open-frame : bloc moteur métallique (#AA8844) avec axe central, bornes et dissipateur. Géométrie stricte, reflets métalliques francs. 64×64.";
d.nano_pack = "Nano-Pack — Conteneur Phase 5. Vue 3/4 : boîte hexagonale verte (#88DD88) avec motif moléculaire central, coque fermée. Géométrie stricte, symétrie parfaite. 64×64.";
d.neural_interface = "Interface Neurale — Composant ultime Phase Finale. Vue 3/4 : boîtier violet (#7E57C2) avec écran central et connexions neurales rayonnantes. Géométrie stricte, coque fermée. 64×64.";
d.neural_map = "Carte Neurale — Composé biologique Phase 6. Vue 3/4 : forme de cerveau bleue (#42A5F5) avec réseaux de connexions synaptiques. Forme organique lisse, couleur vibrante. 64×64.";
d.organic_compound = "Composé Organique — Molécule Phase 6. Vue 3/4 : structure moléculaire verte (#66BB6A) avec atomes reliés par des liaisons. Forme organique arrondie. Couleur vibrante. 64×64.";
d.petroleum_gas = "Gaz Pétrolier — Ressource raffinée Phase 3. Vue 3/4 : flamme de gaz orange (#664422) avec cœur blanc vif. Couleur vive simulant l'activité. 64×64.";
d.pipe = "Tuyau — Composant plomberie Phase 2. Vue 3/4 : section de conduite métallique (#557788) avec brides d'extrémité boulonnées. Géométrie stricte, reflets métalliques. 64×64.";
d.planks = "Planches — Ressource transformée Phase 0. Vue 3/4 : trois planches de bois (#A0724A) juxtaposées avec cernes et nœud visible. Aspect artisanal brut. 64×64.";
d.plant_fiber = "Fibre Végétale — Ressource naturelle Phase 0. Vue 3/4 : fagot de tiges vertes (#5A8C3C) liées ensemble, forme organique allongée. Aspect naturel brut. 64×64.";
d.plastic = "Plastique — Ressource raffinée Phase 3. Vue 3/4 : contenant blanc (#EEEEBB) avec bouchon et lignes de niveau. Contenant standard, centre coloré. 64×64.";
d.processor = "Processeur — Composant électronique Phase 5. Vue 3/4 open-frame : puce verte (#33BB33) carrée avec pins métalliques sur les quatre côtés. Géométrie stricte, symétrie parfaite. Reflets métalliques. 64×64.";
d.protein = "Protéine — Composé biologique Phase 6. Vue 3/4 : structure protéique orange (#FF7043) repliée, forme organique arrondie lisse. Couleur vibrante qui détonne avec l'industrie. 64×64.";
d.rope = "Corde — Ressource tressée Phase 0. Vue 3/4 : bobine de fibre (#8B7355) tressée en cercle avec extrémité effilochée. Aspect artisanal brut. 64×64.";
d.sand = "Sable — Ressource naturelle Phase 1. Vue 3/4 : monticule de grains fins beiges (#DDCC88). Forme organique irrégulière, aspect rugueux. 64×64.";
d.scrap_metal = "Ferraille — Ressource de base Phase 0. Vue 3/4 : morceau de métal tordu (#887766) aux bords déchiquetés, taches de rouille et facettes. Contraste : vestige ruiné du monde d'avant. 64×64.";
d.screw = "Vis — Composant d'assemblage Phase 2. Vue 3/4 : tige filetée métallique (#AAAAAA) avec tête plate. Géométrie stricte, reflets métalliques francs. 64×64.";
d.silicon = "Silicium — Ressource raffinée Phase 1. Vue 3/4 : plaquette ronde (#AACCDD) de silicium avec circuits intégrés en grille et motif de pistes. Géométrie stricte, symétrie parfaite. 64×64.";
d.steam = "Vapeur — Ressource fluide Phase 2. Vue 3/4 : nuage de vapeur blanc-bleuté (#CCDDEE) avec petites bulles. Couleur vive simulant l'activité. 64×64.";
d.steel = "Acier — Métal avancé Phase 2. Vue 3/4 : plaque d'acier brossé (#666688) avec trous de boulons aux coins, lignes de métal et rivets. Géométrie stricte, symétrie parfaite. Reflets métalliques. 64×64.";
d.stem_cells = "Cellules Souches — Culture cellulaire Phase 6. Vue 3/4 : cellule jaune (#FFD54F) avec division visible (lignes de mitose). Forme organique arrondie lisse. Couleur vibrante. 64×64.";
d.stone = "Pierre — Ressource naturelle Phase 0. Vue 3/4 : roche grise (#888888) irrégulière avec facettes d'ombre, fissures. Forme organique asymétrique rugueuse. 64×64.";
d.stone_axe = "Hache en Pierre — Outil Phase 0. Vue 3/4 : tête de pierre (#8B5E3C) taillée attachée à un manche en bois par des liens primitifs. Aspect brut, contraste avec les métaux polis. 64×64.";
d.stone_blade = "Lame en Pierre — Outil Phase 0. Vue 3/4 : lame tranchante en pierre (#887766), forme de couteau/sagaie. Aspect brut artisanal. 64×64.";
d.stone_brick = "Brique de Pierre — Ressource transformée Phase 0. Vue 3/4 : assemblage de briques grises (#887766) rectangulaires avec jointures. Géométrie stricte, aspect taillé. 64×64.";
d.stone_pickaxe = "Pioche en Pierre — Outil Phase 0. Vue 3/4 : tête de pierre (#8B7355) pointue sur manche en bois, liée par des liens primitifs. Aspect brut, contraste avec les métaux. 64×64.";
d.sulfur = "Soufre — Ressource minière Phase 3. Vue 3/4 : cristaux jaunes (#DDDD44) facettés en forme de pointe avec éclat lumineux. Forme géométrique asymétrique. 64×64.";
d.synthetic_blood = "Sang Synthétique — Fluide biologique Phase 6. Vue 3/4 : goutte rouge sombre (#EF5350) avec reflet, contenu dans un récipient standard. Couleur vibrante. 64×64.";
d.synthetic_heart = "Cœur Synthétique — Organe artificiel Phase Finale. Vue 3/4 : forme de cœur rouge (#EF5350) avec ligne de suture médiane et connexions tubulaires. Forme organique lisse, couleur vive. 64×64.";
d.water = "Eau — Ressource fluide Phase 2. Vue 3/4 : bouteille bleue (#3399DD) avec bouchon et niveau d'eau visible. Contenant standard, centre coloré. 64×64.";
d.wood = "Bois — Ressource naturelle Phase 0. Vue 3/4 : bûche (#8B5E3C) cylindrique avec cernes concentriques sur la tranche et écorce. Forme organique, aspect naturel brut par opposition aux métaux usinés. 64×64.";

// ========== BÂTIMENTS ==========

d.aerial_belt = "Convoyeur Aérien — Bâtiment logistique Phase 4. Vue 3/4 open-frame : tapis roulant suspendu (#88AACC) avec structure en U inversé et rouleaux aux extrémités. Zone supérieure (75%) : tapis de transport à ciel ouvert. Façade inférieure (25%) : piliers de support. Transporte les items au-dessus des autres bâtiments. Ports d'entrée/sortie aux bords de tuile. 64×64.";
d.anvil = "Enclume — Bâtiment forge Phase 1. Vue 3/4 open-frame : enclume en métal (#666666) avec corne à gauche, base large, surface de frappe polie. Zone supérieure (75%) : la masse de l'enclume elle-même, visible. Façade inférieure (25%) : socle de support. Reflets métalliques francs. 64×64.";
d.assembler = "Assembleur — Bâtiment production Phase 3. Vue 3/4 open-frame : machine d'assemblage (#4D99CC) avec tapis convoyeur en entrée/sortie, chambre transparente au centre révélant un bras robotique. Zone supérieure (75%) : mécanisme d'assemblage visible. Façade inférieure (25%) : bâti et convoyeur. Voyant vert actif. 128×64.";
d.assembly_crane = "Grue d'Assemblage — Bâtiment production lourde Phase 4. Vue 3/4 open-frame : portique métallique (#3377AA) avec pont roulant et palan central. Zone supérieure (75%) : poutre de levage et chariot mobile. Façade inférieure (25%) : piliers de soutien. Structure ouverte laissant voir les câbles et engrenages. 192×128.";
d.battery_station = "Station de Batteries — Bâtiment production Phase 4. Vue 3/4 open-frame : bloc vert (#33AA33) avec cercles concentriques sur le dessus et bornes métalliques. Zone supérieure (75%) : cellules de batterie visibles. Façade inférieure (25%) : boîtier de connexion. Voyant vert actif. 64×64.";
d.belt = "Convoyeur — Bâtiment logistique Phase 2. Vue 3/4 open-frame : tapis roulant gris (#808080) avec rouleaux cylindriques aux extrémités. Zone supérieure (75%) : surface de transport visible. Façade inférieure (25%) : structure porteuse. Ports d'entrée/sortie aux bords de tuile. 64×64.";
d.bio_lab = "Bio-Laboratoire — Bâtiment production Phase 6. Vue 3/4 : laboratoire vert (#66BB6A) avec coque fermée et dôme en verre pur laissant voir des cultures cellulaires vibrantes (rouges/rosées). Base strictement industrielle en acier froid. Encapsule des fluides biologiques derrière des vitres. Voyant vert actif. 128×128.";
d.bio_printer = "Bio-Imprimante — Bâtiment production Phase 6. Vue 3/4 : imprimante 3D biologique (#4DB6AC) avec coque fermée et chambre d'impression vitrée. Base en acier froid. Fluides vibrants visibles à travers les vitres. Voyant vert actif. 128×128.";
d.blast_furnace = "Haut Fourneau — Bâtiment production Phase 2. Vue 3/4 open-frame : four industriel (#AA4422) avec cuve effilée verticale, anneaux de renfort métalliques et trou de coulée rougeoyant (#FF6622). Zone supérieure (75%) : la cuve ouverte laisse voir la matière en fusion à l'intérieur. Façade inférieure (25%) : base en briques et buses d'aération. Couleurs vives pour simuler l'activité. 128×128.";
d.burner_generator = "Générateur Charbon — Bâtiment énergétique Phase 1. Vue 3/4 open-frame : générateur orange (#DD6622) robuste avec grille d'aération, jauge de pression et tuyau d'échappement. Zone supérieure (75%) : mécanisme de combustion visible (charbon + flammes). Façade inférieure (25%) : socle et sortie d'énergie. Bandes d'avertissement jaunes. 128×64.";
d.campfire = "Feu de Camp — Bâtiment cuisson Phase 0. Vue 3/4 : cercle de pierres grises (#666) avec flammes orangées (#FF6622) et cœur jaune vif (#FFDD44). Zone supérieure (75%) : les flammes visibles. Façade inférieure (25%) : les pierres qui les retiennent. Contraste : artisanal, primitif. Étincelles. 64×64.";
d.chemical_lab = "Laboratoire Chimique — Bâtiment production Phase 3. Vue 3/4 open-frame : laboratoire violet (#664488) avec ballons, fioles et tubes à essai interconnectés. Zone supérieure (75%) : réactions chimiques visibles dans les cuves (liquides colorés vifs). Façade inférieure (25%) : paillasse et rangement. Structure ouverte, toutes les manipulations sont visibles. 128×128.";
d.compactor = "Compacteur — Bâtiment utilitaire Phase 5. Vue 3/4 : bloc beige (#AAAA77) avec cavité centrale de compression et coque fermée. Géométrie stricte, lignes de force métalliques. Voyant vert actif. 64×64.";
d.deep_core_drill = "Foreuse Profonde — Bâtiment minier Phase 5. Vue 3/4 : énorme plateforme de forage (#664433) avec mât central vertical, base industrielle massive et structure en treillis. Coque fermée avec soupapes et conduits de forage. Extraie les minerais sans épuiser le dépôt. Voyant vert actif. 320×320.";
d.electric_generator = "Générateur Électrique — Bâtiment énergétique Phase 3. Vue 3/4 open-frame : générateur moderne (#FFAA33) avec ailettes de refroidissement horizontales, panneau de contrôle (manomètre), bornes électriques (+/-) et voyant vert. Zone supérieure (75%) : stator et rotor visibles. Façade inférieure (25%) : base de montage. 128×64.";
d.electronics_lab = "Laboratoire d'Électronique — Bâtiment production Phase 4. Vue 3/4 open-frame : laboratoire vert-bleu (#33AA88) avec quatre postes de travail, circuits et interconnexions visibles. Zone supérieure (75%) : les circuits en cours d'assemblage. Façade inférieure (25%) : établis de travail. 128×128.";
d.furnace = "Four — Bâtiment production Phase 1. Vue 3/4 open-frame : four en briques (#884422) avec chambre de combustion visible, flammes vives (#FF6622), grille en bas et cheminée. Zone supérieure (75%) : la chambre ouverte montre la combustion et le métal en fusion. Façade inférieure (25%) : cendrier et arrivée d'air. Contraste avec les ruines : briques bien alignées, métal propre. 128×64.";
d.gear_press = "Presse à Engrenages — Bâtiment production Phase 2. Vue 3/4 open-frame : presse mécanique (#887766) avec grand engrenage denté comme signature visuelle, vérin de pression vertical et enclume. Zone supérieure (75%) : l'engrenage et le mécanisme de pressage. Façade inférieure (25%) : socle et contre-plaque. Reflets métalliques. 64×64.";
d.high_speed_belt = "Convoyeur Haute Vitesse — Bâtiment logistique amélioré Phase 5. Vue 3/4 open-frame : tapis roulant orange (#CC8844) avec rouleaux larges aux extrémités et bande de signalisation jaune. Zone supérieure (75%) : tapis visible. Façade inférieure (25%) : structure renforcée. Transport 3x plus rapide. Ports d'entrée/sortie aux bords de tuile. 64×64.";
d.manual_miner = "Mineur Manuel — Bâtiment minier Phase 1. Vue 3/4 open-frame : trépied métallique (#AA7733) avec tête de forage triangulaire pointée vers le bas. Zone supérieure (75%) : mécanisme de battement visible. Façade inférieure (25%) : embase au sol. Extrait les minerais d'un dépôt. 64×64.";
d.motor_foundry = "Fonderie de Moteurs — Bâtiment production Phase 4. Vue 3/4 open-frame : fonderie beige (#AA8844) avec deux moteurs en cours d'assemblage sur des postes séparés, cuve de coulée centrale. Zone supérieure (75%) : les moteurs ouverts montrent leurs bobines de cuivre. Façade inférieure (25%) : socle de travail. 128×128.";
d.nanite_assembler = "Assembleur Nanite — Bâtiment production Phase 5. Vue 3/4 : assemblage circulaire turquoise (#44DDBB) avec coque fermée et dôme vitré laissant voir des nano-particules en orbite autour d'un centre lumineux. Base en acier froid. Géométrie parfaitement symétrique. Voyant vert actif. 128×128.";
d.oil_pump = "Pompe à Pétrole — Bâtiment extracteur Phase 3. Vue 3/4 open-frame : plateforme de pompage gris foncé (#444455) avec balancier oscillant et tête de puits. Zone supérieure (75%) : le balancier et la mécanique de pompage visibles. Façade inférieure (25%) : base et raccordement au gisement. 128×128.";
d.pipe = "Tuyau — Bâtiment transport fluide Phase 2. Vue 3/4 : conduite horizontale (#557788) avec brides boulonnées aux extrémités et renforts. Pose par cliqué-glissé. Ports de connexion touchant les bords de tuile. 64×64.";
d.power_pole = "Pylône — Bâtiment électrique Phase 3. Vue 3/4 : poteau gris (#888888) avec traverse horizontale, isolateurs suspendus aux extrémités et base en béton. Structure ouverte minimaliste. 64×64.";
d.scanner_array = "Réseau de Scanners — Bâtiment production Phase 6. Vue 3/4 : grande antenne parabolique bleue (#42A5F5) avec cercles concentriques rayonnant du centre, coque fermée technique. Base en acier froid. Voyant vert actif. 192×192.";
d.sorter = "Trieur — Bâtiment logistique Phase 4. Vue 3/4 open-frame : module hexagonal vert (#66AA66) avec flèche directionnelle interne. Zone supérieure (75%) : mécanisme de tri visible. Façade inférieure (25%) : base. Ports d'entrée/sortie aux bords de tuile. 64×64.";
d.splitter = "Séparateur — Bâtiment logistique Phase 2. Vue 3/4 open-frame : module jaune (#AAAA00) en forme de croix avec séparateur central en X. Zone supérieure (75%) : les quatre bras de convoyage. Façade inférieure (25%) : base. Ports d'entrée/sortie sur les quatre côtés. 64×64.";
d.steam_generator = "Générateur Vapeur — Bâtiment énergétique Phase 2. Vue 3/4 open-frame : chaudière industrielle (#CCDDEE) avec foyer roueoyant à gauche, faisceau de tubes bouilleurs au centre, soupape de sécurité, sortie vapeur et manomètre. Zone supérieure (75%) : la chaudière ouverte montre le bouillonnement de l'eau et les flammes. Façade inférieure (25%) : grille et cendrier. Produit vapeur et électricité. 192×64.";
d.storage_chest = "Coffre de Stockage — Bâtiment utilitaire Phase 3. Vue 3/4 : coffre en bois jaune (#CC9900) avec couvercle bombé, ferrures métalliques aux coins, cadenas central et rivets. Zone supérieure (75%) : couvercle fermé. Façade inférieure (25%) : paroi avant. Contraste : bois propre, métal bien entretenu. 64×64.";
d.synthesizer = "Synthétiseur — Bâtiment production Phase 6. Vue 3/4 : module orange (#FF7043) avec coque fermée et trois cuves de réaction vitrées interconnectées par des tubes. Base en acier froid. Fluides vibrants visibles à travers les vitres. 64×64.";
d.tissue_cultivator = "Cultivateur de Tissus — Bâtiment production Phase 6. Vue 3/4 : cuve de culture violette (#AB47BC) avec coque fermée et dôme vitré révélant des cellules en division. Base industrielle en acier. Encapsule des fluides biologiques vibrants derrière une vitre. 128×128.";
d.water_pump = "Pompe à Eau — Bâtiment extracteur Phase 2. Vue 3/4 open-frame : station de pompage bleue (#3399DD) avec réservoir horizontal, cylindre vertical, piston, manomètre et tuyaux d'entrée/sortie. Zone supérieure (75%) : le mécanisme de pompage visible. Façade inférieure (25%) : embase. Ports de connexion aux bords de tuile. 128×128.";
d.workbench = "Établi — Bâtiment craft Phase 0. Vue 3/4 open-frame : table de travail en bois (#8B5E3C) avec outils (marteau, pièces métalliques) posés sur le plateau, tiroir de rangement. Zone supérieure (75%) : plan de travail visible. Façade inférieure (25%) : piètement en bois. Contraste : artisanal, primitif. 64×64.";

// ========== CAPSULES ==========

d.genesis_capsule = "Capsule Genesis — Bâtiment narratif central Phase 0 (Réveil). Vue 3/4 : capsule cryo ovale verticale, vitre noire opaque, coque fissurée et envahie de débris. Aucun signe de vie. Elle est l'anomalie parfaite : métal blanc épuré perdu au milieu des ruines végétalisées. 256×256.";
d.genesis_capsule_t0 = "Capsule Genesis Phase 1 (Étincelle). Débris retirés, coque propre. Un premier voyant vert s'allume sur le côté. La vitre reste noire mais l'espoir renaît. 256×256.";
d.genesis_capsule_t1 = "Capsule Genesis Phase 2 (Rouille &amp; Vapeur). Condensation visible sur la vitre. Un liquide de refroidissement sombre circule dans des canalisations externes. Bulles. 256×256.";
d.genesis_capsule_t2 = "Capsule Genesis Phase 3 (Fil du Cuivre). Faible lueur interne derrière la vitre. Le nouveau noyau CPU émet une lumière bleutée. Voyants vert et rouge actifs. 256×256.";
d.genesis_capsule_t3 = "Capsule Genesis Phase 4 (Pouls). Rétroéclairage rythmé simulant un battement cardiaque. La lumière pulse doucement. Silhouette encore indistincte. 256×256.";
d.genesis_capsule_t4 = "Capsule Genesis Phase 5 (Nanites). Coque lissée, vitre partiellement transparente révélant une vague silhouette humaine. Teintes argentées et blanches dominent. 256×256.";
d.genesis_capsule_t5 = "Capsule Genesis Phase 6 (Genèse). Vitre s'éclaircit, teintes biologiques rosées/rouges à l'intérieur. Une main est distinctement visible contre le verre de l'intérieur. 256×256.";
d.genesis_capsule_t6 = "Capsule Genesis Phase 7 (Premier Souffle). Tous les systèmes au vert. Vitre totalement transparente. La main appuie contre la paroi. L'être à l'intérieur est prêt à naître. 256×256.";
d.genesis_capsule_t7 = "Capsule Genesis Phase Finale. Apothéose lumineuse. La capsule irradie une lumière blanche chaude. La vitre s'ouvre. 256×256.";

// ========== EXTRA ==========
d.electronic_module = "Module Électronique — Composant Phase 4. Vue 3/4 open-frame : petite puce verte (#66CC66) carrée avec quatre zones de connexion. Géométrie stricte, symétrie parfaite. Reflets métalliques francs. 64×64.";

// --------------------------------------------------------------------------

function updateSvg(filePath, stem) {
  let svg = fs.readFileSync(filePath, "utf8");
  const desc = d[stem];
  if (!desc) { console.log(`  ⚠ ${stem} — pas de description`); return; }

  const name = stem
    .replace(/^genesis_capsule_t(\d)/, "Capsule Genesis T$1")
    .replace(/^genesis_capsule$/, "Capsule Genesis")
    .replace(/_/g, " ")
    .replace(/\b\w/g, c => c.toUpperCase());

  const title = `${name}`;

  // Remove existing title, desc, and comments
  svg = svg.replace(/  <title>.*<\/title>\n?/g, "");
  svg = svg.replace(/  <desc>.*<\/desc>\n?/g, "");
  svg = svg.replace(/  <!--.*-->\n?/g, "");

  // Insert new title + desc right after <svg ...>
  svg = svg.replace(/(<svg[^>]*>)/, `$1\n  <title>${title}</title>\n  <desc>${desc}</desc>`);

  fs.writeFileSync(filePath, svg);
  console.log(`  ✓ ${stem}_base.svg`);
}

const files = fs.readdirSync(MANUAL_DIR).filter(f => f.endsWith("_base.svg"));
console.log(`Mise à jour des descriptions (${files.length} SVGs)...\n`);
for (const file of files) {
  const stem = file.replace("_base.svg", "");
  updateSvg(path.join(MANUAL_DIR, file), stem);
}
console.log(`\nTerminé !`);
