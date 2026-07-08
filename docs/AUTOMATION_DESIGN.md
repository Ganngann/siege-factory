# Automation Design

## Vision
Post-apocalyptic. Dernier survivant. Une capsule cryo brisée est visible dès le début — silencieuse, vitre opaque, compteurs éteints. L'objectif unique du jeu (~200h) : la réparer assez pour créer le premier humain. On ne sait pas s'il y en a d'autres.

Le joueur ne sait pas ce qu'il faut faire au début. Il découvre en produisant (système de découverte). Chaque palier de la capsule est un mystère à débloquer.

C'est un jeu d'automation, pas un tower defense.

## Principes
- Chaque bâtiment/mécanique apparaît seulement quand le joueur comprend son usage
- Menu : bâtiments indisponibles cachés ; catégories vides cachées ; catégorie à 1 bâtiment → le bâtiment directement
- La capsule est toujours visible dans le monde et s'illumine à chaque palier
- Découvertes fréquentes (~toutes les 30 min)

## Progression (200h)

| Segment | Contenu | Durée |
|---------|---------|-------|
| **Phase 0** | Outils pierre, Workbench, Campfire, crafts manuels | 2-5h |
| **Phase 1** | Énergie charbon, extracteurs, fours, premiers lingots | 10-15h |
| **Phase 2** | Fluides, vapeur, acier, belts, transport automatisé | 15-20h |
| **Phase 3** | Électricité, circuits, chimie (pétrole, plastique) | 20-30h |
| **Phase 4** | Moteurs, batteries, logistique avancée | 30-40h |
| **Phase 5** | Nano-assemblage, forage profond, composites | 40-50h |
| **Phase 6** | Bio-ingénierie, tissus, synthèse protéines | 40-50h |
| **Phase finale** | Assemblage des 4 composants → premier humain | ~20h |

La fin du jeu est la création du premier humain. Pas de colonie, pas d'après.

## Capsule (progression visuelle)

| Palier | État capsule | Déblocage |
|--------|-------------|-----------|
| 0 | Éteinte, vitre noire | Outils pierre, dégager les débris |
| 1 | Voyants d'alimentation verts | Énergie charbon, extracteurs, fours |
| 2 | Liquide visible, bulles | Fluides, vapeur, acier, belts |
| 3 | Lueur interne faible | Électricité, circuits, chimie |
| 4 | Battement cardiaque (son + lumière) | Moteurs, batteries, logistique |
| 5 | Vitre partiellement transparente, silhouette | Nanites, forage profond |
| 6 | Signes vitaux stables, main visible | Bio-ingénierie, tissus |
| Final | Vitre claire, main contre la paroi → **naissance** | Assemblage 4 composants |
