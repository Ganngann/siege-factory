# Genesis Protocol — Guide de style visuel (SVG vers PNG)

## 1. Univers et Traduction Visuelle

**Cadre** : Terre post-apocalyptique, ~2147. Une civilisation avancée s'est effondrée.

**Tonalité** : Mélancolique mais pas glauque. L'activation d'une machine est une victoire silencieuse.

**La Grande Dichotomie Visuelle :**
- **Le Monde / Les Ruines** : La nature a repris ses droits. Routes brisées, supermarchés effondrés. Dominance du lierre, de la mousse et de la rouille.
- **Les Constructions du Joueur** : Elles doivent contraster fortement avec les ruines. Elles sont propres, métalliques et lumineuses. Le joueur ramène l'ordre et la technologie dans un monde sauvage. *(Règle stricte : aucune végétation sur les usines du joueur).*
- **L'Épicentre (La Capsule)** : Le cœur narratif. Design épuré, métal blanc, verre fumé. C'est l'anomalie parfaite au milieu des ruines végétalisées au début du jeu.

## 2. Conventions Graphiques (Rendu PNG final depuis SVG)

**Vue (Perspective)** : Orthogonale 3/4 (Caméra inclinée sur grille droite).
- La grille du jeu reste strictement carrée (pas de tuiles en diagonale).
- Les objets montrent leur façade avant (Sud) et leur partie supérieure (voir règle "Open-top").

**Contraintes de tuile (Pas de débordement)** :
- Boîte stricte : Le sprite dessiné ne doit jamais déborder de sa taille logique en jeu (1×1 = 64×64 px). Tout doit rentrer dans la zone allouée (pas de Y-sorting).

**Ratio 3/4 (Façade / Cœur)** :
- Pour créer l'illusion de volume, réservez le quart inférieur (~25% de la hauteur, soit ~16px) pour la façade avant, et les trois quarts supérieurs (~75%, soit ~48px) pour la zone d'opération de la machine.

**Marges de lisibilité** :
- Laissez une marge vide de 1 à 2 pixels sur les bords de la machine. Seuls les "ports" de connexion (tuyaux, entrées/sorties de belts) ont le droit de toucher le bord absolu des 64×64px.

**Échelle** : Tuile de base de 64×64 pixels. Les bâtiments occupent de 1×1 (64×64) à 5×5 (320×320).

**Contours (Outlines)** : #333333 (ou une version très assombrie de la couleur de base). Épaisseur 0.5 à 1.0. Pas de contours sur les ombres ou les reflets.

**Ombrage et Éclairage (SVG statique)** :
- Ombre portée : Ellipse noire semi-transparente rgba(0,0,0, 0.10-0.15) sous la base de chaque objet, confinée strictement dans les limites du sprite.
- Reflets : Uniquement sur les surfaces métalliques. Lignes blanches semi-transparentes sur les arêtes.
- Voyants lumineux : Petits cercles francs (Vert = Actif, Rouge = Arrêt). Pour palier l'absence d'animation, les parties fonctionnelles (cœur du four, liquides) doivent avoir des couleurs vives pour simuler l'activité.

## 3. Palette de Couleurs (Hex)

**Le Monde (Ruines & Nature)** :
- Rouille & Oxydation : #AA4422, #CC6633, #DD6622, #FFAA33
- Végétation usée : #5A8C3C, #66BB6A, #4DB6AC
- Ruines (Béton/Pierre) : #888888, #999999, #666666

**Le Joueur (Usine & Outils)** :
- Outils Phase 0 : Mélange de primitif (Bois/Pierre) et de vestiges (câbles arrachés, bouts de tôle blanche de la capsule).
- Métaux d'usine : #887766 (Fer basique), #666688, #8888AA (Acier brossé)
- Cuivre & Électronique : #CC8844, #D68A4C

**Indicateurs & Éléments Vifs (L'éveil de l'usine)** :
- Circuits / Voyants Actifs : #33BB33
- Eau / Refroidissement : #3399DD
- Feu / Vapeur / Chaleur : #FF6622
- Biologie (Phase 6+) : #D92525 (Sang), #E5989B (Biomasse), #9B59B6 (Enzymes)

## 4. Direction Artistique des Éléments

### La Capsule (Genesis Array) — L'indicateur Narratif

Taille : 256×256 px (4×4 tuiles). Forme ovale verticale.
Concept : L'état évolue visuellement (8 paliers).

- **Phase 0 (Réveil)** : Éteinte, fissurée, envahie de débris, vitre noire opaque.
- **Phase 1 (Étincelle)** : Débris retirés, quelques voyants d'alimentation verts s'allument.
- **Phase 2 (Rouille & Vapeur)** : Liquide de refroidissement visible, bulles, condensation sur la vitre.
- **Phase 3 (Fil du Cuivre)** : Lueur interne faible due au nouveau noyau informatique (CPU).
- **Phase 4 (Pouls)** : Rétroéclairage rythmé simulant un battement cardiaque (via voyants ou lueur).
- **Phase 5 (Nanites)** : Coque lissée, vitre partiellement transparente révélant une vague silhouette.
- **Phase 6 (Genèse)** : Vitre claire, teintes biologiques (rosées/rouges) à l'intérieur, signes vitaux stables, une main est distinctement visible contre le verre.
- **Final (Premier Souffle)** : Tous les systèmes au vert, vitre totalement transparente, la main bouge.

### Bâtiments et Usines (L'évolution du Joueur)

**Design "Open-Top" (Sans toit) et Narration** : Vu en 3/4, la zone supérieure de la machine (les 75% du haut) remplace un simple toit plat pour montrer directement sa fonction :

- **Phases 0-4 (Mécanique brute)** : Machines totalement ouvertes (Open-frame). Pas de toit. Le joueur voit directement les engrenages, la lave dans les fours, les bobines de cuivre, les courroies.
- **Phases 5-6 (Stérilité & Biologie)** : Les technologies nanites et biologiques nécessitent la pureté. Ces machines ont des coques fermées ou des dômes en verre pur. Elles conservent une base strictement industrielle (cadres en acier froid) mais encapsulent leurs fluides vibrants (rouge/violet) derrière des vitres.
- **Lisibilité** : Les ports d'entrée/sortie (belts/pipes) doivent s'aligner parfaitement avec le bord absolu de la tuile.

### Ressources (Items d'inventaire)

Taille : 64×64 px (sans fond).

Typologie :
- **Naturelles/Minerais** : Formes organiques, asymétriques, aspect rugueux.
- **Métaux/Composants** : Géométrie stricte, symétrie parfaite, reflets métalliques francs.
- **Fluides** : Contenus dans des récipients standards avec un centre coloré.
- **Biologiques (Phase 6)** : Formes organiques mais lisses (arrondies, cellulaires), couleurs vibrantes qui détonnent avec l'industrie.
