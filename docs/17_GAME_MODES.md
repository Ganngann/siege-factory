# Game Modes — Siege Factory

## Architecture

Les modes de jeu sont gérés par l'enum `GameState` :

```
Menu → Playing → GameOver → Menu
           ↓
        Loading → Playing
```

Chaque état active/désactive ses systèmes.
