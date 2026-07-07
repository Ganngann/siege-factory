# UI / HUD — Siege Factory

## Principes

- UI découplée de la logique : les systèmes UI lisent des Resources, ne modifient pas l'état de jeu directement
- Les actions utilisateur envoient des Events que les systèmes de logique traitent
- Pas d'ECSs dans l'UI (sauf composants UI Bevy)

## Règles

- L'UI ne modifie jamais directement un component ECS de jeu
- L'UI lit des Resources publiques, envoie des Events
- Les systèmes de logique traitent les Events, mettent à jour les Resources si nécessaire
- Le rendu est géré par Bevy (Query standard), pas par l'UI
