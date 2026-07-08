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
- **Highlight :** La capsule est visible, massive, complètement éteinte
- **Toast :** *"That structure looks important — go check it out."*
- **Attendre :** Joueur s'approche à ≤3 tuiles de la capsule

### Step 3 — Scavenge
- **Highlight :** Points de collecte au sol (scrap_metal, wood, stone, clay) clignotent
- **Toast :** *"Pick up resources from the ground. You'll need them to survive."*
- **Attendre :** Joueur ramasse ≥3 items

### Step 4 — Craft stone tools
- **Toast :** *"Open the crafting menu (C) and craft a stone axe, a stone pickaxe, and a stone blade."*
- **Attendre :** Joueur craft les 3 outils

### Step 5 — Chop wood & mine stone
- **Highlight :** Arbre proche + rocher proche
- **Toast :** *"Use the axe on trees (E) and the pickaxe on stone to gather resources faster."*
- **Attendre :** Joueur récolte ≥5 wood + ≥5 stone

### Step 6 — Gather clay & plant fiber
- **Highlight :** Dépôt d'argile + herbes hautes proches
- **Toast :** *"Collect clay from the ground and use the blade to cut plant fiber from vegetation."*
- **Attendre :** Joueur récolte ≥4 clay + ≥4 plant_fiber

### Step 7 — Build a Workbench
- **Toast :** *"Open the build menu (B) and place a Workbench. This is your first building."*
- **Attendre :** Workbench construit

### Step 8 — Craft materials at the Workbench
- **Toast :** *"Use the Workbench to process raw resources: scrap_metal → iron_parts, wood → planks, stone → stone_brick, clay → ceramic, plant_fiber → rope."*
- **Attendre :** Joueur craft ≥1 de chaque matériau

### Step 9 — Build a Campfire
- **Toast :** *"Now place a Campfire next to your Workbench. It lets you smelt and fire materials."*
- **Attendre :** Campfire construit

### Step 10 — Smelt your first iron_ingot
- **Toast :** *"Use the Campfire to smelt iron_parts + coal → iron_ingot. This is your first metal."*
- **Attendre :** Joueur craft ≥1 iron_ingot

### Step 11 — Clear the capsule debris
- **Highlight :** La capsule clignote
- **Toast :** *"The capsule is covered in debris. Use your hammer and pickaxe to clear the panels and inspect it."*
- **Attendre :** Joueur interagit avec la capsule, les panneaux s'ouvrent

### Step 12 — The capsule needs power
- **Animation :** L'écran de la capsule affiche *"ALIMENTATION REQUISE"*
- **Toast :** *"The capsule is alive — it needs power. Build a Burner Generator next to it to restore energy. You're on your own now."*
- **Fin du tutorial**

## Technical Notes
- `TutorialState { current_step: usize, completed: bool }`
- Steps vérifiés par conditions dans des systèmes `RunIf`
- Highlights : `TileHighlight` ou sprite enfant temporaire
- Après `completed = true`, plus aucun step ne se déclenche
- `completed` est sauvegardé (ne pas rejouer le tutorial sur une save)
