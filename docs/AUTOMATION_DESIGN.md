# Automation Design

## Vision
Post-apocalyptic. Dernier survivant. Une capsule cryo brisée est visible dès le début — silencieuse, vitre opaque, compteurs éteints. L'objectif unique du jeu (~200h) : la réparer assez pour créer le premier humain. On ne sait pas s'il y en a d'autres.

Le joueur ne sait pas ce qu'il faut faire au début. Il découvre en produisant (système de découverte). Chaque palier de la capsule est un mystère à débloquer.

C'est un jeu d'automation, pas un tower defense. Les vagues sont un scaffold.

## Principes
- Chaque bâtiment/mécanique apparaît seulement quand le joueur comprend son usage
- Menu : bâtiments indisponibles cachés ; catégories vides cachées ; catégorie à 1 bâtiment → le bâtiment directement
- La capsule est toujours visible dans le monde et s'illumine à chaque palier
- Découvertes fréquentes (~toutes les 30 min)

## Progression (200h)

| Segment | Contenu | Durée |
|---------|---------|-------|
| **Phase 0** | Établi, four manuel, outils, premiers crafts | 2-5h |
| **Phase 1** | Alimentation capsule → débloque énergie, extracteurs | 10-15h |
| **Phase 2** | Purification eau → débloque chimie, fonderies | 15-20h |
| **Phase 3** | Composés organiques → débloque bio-lab, belts, logistique | 20-30h |
| **Phase 4** | Synthèse protéines → usine complexe, multi-bâtiments | 30-40h |
| **Phase 5** | Nanites de reconstruction → automation lourde, upgrades | 40-50h |
| **Phase 6** | Bio-impression 3D → bots logistiques, forage profond | 40-50h |
| **Phase finale** | Activation capsule → premier humain | ~20h |

La fin du jeu est la création du premier humain. Pas de colonie, pas d'après.

## Capsule (progression visuelle)

| Palier | État capsule | Déblocage |
|--------|-------------|-----------|
| 0 | Éteinte, vitre noire | — |
| 1 | Voyants d'alimentation verts | Énergie, extracteurs |
| 2 | Liquide visible, bulles | Chimie, fonderie |
| 3 | Lueur interne faible | Bio-lab, belts |
| 4 | Battement cardiaque (son + lumière) | Synthèse protéines |
| 5 | Vitre partiellement transparente, silhouette | Nanites, upgrades |
| 6 | Capteurs vitaux affichent des signes | Bio-impression, bots |
| Final | Vitre claire, main contre la paroi → **naissance** | — |
