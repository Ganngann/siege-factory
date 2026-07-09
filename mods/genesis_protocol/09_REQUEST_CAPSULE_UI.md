# Request 09 — UI de progression pour la Capsule

## Contexte

Quand le joueur interagit avec la capsule genesis_ark (touche E), il voit une fenêtre générique (inventaire de dépôt) qui n'affiche aucune information utile sur la progression.

## Problème

- Aucun affichage du tier actuel (ex: "Tier 0 — Marteau")
- Aucune indication des items requis pour le tier en cours (ex: "Apporte : 1 Marteau")
- Aucun historique des tiers déjà complétés (logs)
- Le joueur ne sait pas quoi donner ni pourquoi

## Demande

Créer une UI spécifique pour les structures à tiers (`Capsule` / `genesis_ark`) qui affiche :

1. **Nom et description du tier actuel** (depuis le log correspondant)
2. **Items requis** avec compteur : ce que le joueur a déjà donné / ce qu'il faut
3. **Progression visuelle** des tiers complétés (puces/barres)
4. **Bouton ou indication pour donner les items** (touche E)

Données disponibles dans les TOML :

```toml
# buildings.toml
[[buildings.genesis_ark.tiers]]
required_items = { hammer = 1 }           # Items nécessaires
log_id = "genesis_phase_0_clear"          # Lien vers le log narratif
texture = "genesis_capsule"               # Visuel du tier

# story/logs.toml
[[logs]]
id = "genesis_phase_0_clear"
tier = 0
title = "Déblayage"
text = "Le marteau frappe les panneaux d'accès..."
```

Format UI suggéré :

```
┌─ CAPSULE GENESIS ─────────────────┐
│                                    │
│  ● Tier 0 : Déblayage (complété)   │
│  ○ Tier 1 : Réveil                 │
│    Items requis :                  │
│    [▰▰▰▰▰▰▰▰▰▰] Corde    3/3    ✓ │
│                                    │
│  [Appuyez sur E pour donner]       │
│                                    │
│  ┌────────────────────────────┐    │
│  │ "La corde s'enroule autour │    │
│  │  des connecteurs..."       │    │
│  └────────────────────────────┘    │
└────────────────────────────────────┘
```

## Résultat attendu

- Le joueur voit immédiatement ce qu'il doit apporter à la capsule
- La progression narrative est lisible
- Fini la fenêtre "inventaire vide" qui ne veut rien dire
