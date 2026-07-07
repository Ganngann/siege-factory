# Game Design — Siege Factory

## Vision

Jeu d'automation : carte infinie, multijoueur, arbre technologique profond, recettes en arborescence, N ressources.

Le mode actuel est un scaffold temporaire qui restera comme mode de jeu une fois la destination atteinte.

## Philosophie

- Le joueur commence manuel et gagne l'automatisation par le jeu
- Tout ce qui est répétitif doit devenir automatisable à terme

## Principe d'évolution

Chaque feature est un investissement pour la destination :

- **ECS** : scale, determinism, découplage
- **Data-driven** (registres TOML) : modding, itération rapide, N ressources
- **Events** (logique ↔ UI) : multi-ready, découplage
- **String IDs** : types dynamiques, flexibilité
