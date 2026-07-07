# Tutorial Design

## Overview
Guide le joueur dans les premières heures (Phase 0). Après le tutorial, le système de découverte prend le relais. Pas d'overlay — toasts + highlights.

La capsule est visible dès le spawn, éteinte. Le joueur ne sait pas ce que c'est.

## Steps

### Step 1 — Move
- **Déclencheur :** `OnEnter(GameState::Playing)`
- **Toast :** *"Use WASD or arrow keys to move."*
- **Attendre :** Joueur se déplace de ≥5 tuiles

### Step 2 — Discover the Capsule
- **Highlight :** La capsule émet une faible lueur, unique dans le paysage
- **Toast :** *"That structure looks important — go check it out."*
- **Attendre :** Joueur s'approche à ≤3 tuiles de la capsule

### Step 3 — Scavenge
- **Highlight :** Points de collecte au sol (cailloux, bois) clignotent
- **Toast :** *"Pick up resources from the ground. You'll need them."*
- **Attendre :** Joueur ramasse ≥3 items

### Step 4 — Craft a pickaxe
- **Toast :** *"Open the crafting menu (C) and make a stone pickaxe."*
- **Attendre :** Joueur craft une pioche

### Step 5 — Mine ore
- **Highlight :** Gisement de minerai le plus proche
- **Toast :** *"Walk to the deposit and hold E to mine ore."*
- **Attendre :** Joueur mine ≥5 minerai

### Step 6 — Find clay & make bricks
- **Highlight :** Gisement d'argile proche
- **Toast :** *"You'll need clay for bricks. Find a clay deposit and mine it."*
- **Attendre :** Joueur craft ≥4 briques

### Step 7 — Build a Workbench
- **Toast :** *"Place a Workbench from the build menu (B). This is your first building."*
- **Attendre :** Établi construit

### Step 8 — Build a Furnace
- **Toast :** *"Now place a Furnace. It needs bricks and stone."*
- **Attendre :** Four construit

### Step 9 — Craft capsule parts
- **Toast :** *"Use your Workbench and Furnace to craft the first capsule components: a power relay and a coolant pipe."*
- **Attendre :** Joueur craft les deux pièces

### Step 10 — Deliver parts to the capsule
- **Toast :** *"Walk up to the capsule and press E to install the parts."*
- **Attendre :** Les deux pièces sont installées

### Step 11 — Capsule lights up
- **Animation :** La capsule s'allume (voyants verts, faible lueur)
- **Toast :** *"The capsule has power. It needs more to function. Explore, discover new recipes, and automate. You're on your own now."*
- **Fin du tutorial**

## Technical Notes
- `TutorialState { current_step: usize, completed: bool }`
- Steps vérifiés par conditions dans des systèmes `RunIf`
- Highlights : `TileHighlight` ou sprite enfant temporaire
- Après `completed = true`, plus aucun step ne se déclenche
- `completed` est sauvegardé (ne pas rejouer le tutorial sur une save)
