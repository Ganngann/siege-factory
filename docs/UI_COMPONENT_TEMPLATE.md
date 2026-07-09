# Template — Créer un composant UI personnalisé

## 1. Créer le fichier Rust

`src/ui/components/mon_composant.rs` :

```rust
use bevy::prelude::*;
use crate::ui::registry::{UiComponent, spawn_child};

pub struct MonComposant;

impl UiComponent for MonComposant {
    fn id(&self) -> &str {
        "mon_composant"  // ← utilisé dans le TOML : { type = "mon_composant" }
    }

    fn render(
        &self,
        commands: &mut Commands,
        parent: Entity,
        config: &toml::Value,        // ← la config TOML de cette instance
        data: &UiDataContext,         // ← accès aux données du jeu
        theme: &Theme,                // ← couleurs/polices du thème
        registry: &ComponentRegistry, // ← pour sous-composants
    ) -> Entity {
        // 1. Lire les paramètres TOML
        let ma_cle = config.get("ma_cle").and_then(|v| v.as_str()).unwrap_or("défaut");

        // 2. Lire les données du jeu
        let valeur = data.resolve("building.name");
        let hp: f32 = data.resolve("hp.current").parse().unwrap_or(0.0);

        // 3. Créer l'UI
        spawn_child(commands, parent, (
            Text::new(format!("{}: {}", ma_cle, valeur)),
            crate::core::game_font::tf(theme.font_size_body),
            TextColor(theme.text_primary),
        ))
    }
}
```

## 2. Déclarer dans `src/ui/components/mod.rs`

```rust
pub mod mon_composant;
```

## 3. Enregistrer dans `src/ui/mod.rs` (UiPlugin)

```rust
use crate::ui::components::mon_composant::MonComposant;

// Dans UiPlugin::build() :
comp_registry.register(Box::new(MonComposant));
```

## 4. Utiliser depuis un TOML

```toml
[[sections]]
type = "mon_composant"
ma_cle = "ma_valeur"
```

## API disponible

### spawn_child

```rust
let enfant = spawn_child(commands, parent, (composant1, composant2, ...));
```

Crée une entité comme enfant du `parent` et retourne son Entity ID.

### UiDataContext::resolve

```rust
let valeur: String = data.resolve("cle");  // retourne String
let nb: f32 = data.resolve("cle").parse().unwrap_or(0.0);
```

Résout une clé de donnée du jeu. Voir `docs/UI_COMPONENTS.md` pour la liste des clés disponibles.

### Ajouter une nouvelle clé de donnée

Dans `src/ui/context.rs`, méthode `resolve()` :

```rust
"ma_nouvelle_cle" => {
    self.get::<MonComposant>(self.entity)
        .map(|c| c.ma_valeur.to_string())
        .unwrap_or_default()
}
```

### Sous-composants

Pour qu'un composant rende des enfants configurables via TOML :

```rust
if let Some(elements) = config.get("elements").and_then(|v| v.as_array()) {
    for el_cfg in elements {
        let cid = el_cfg.get("type").and_then(|v| v.as_str()).unwrap_or("label");
        if let Some(comp) = registry.get(cid) {
            comp.render(commands, parent, el_cfg, data, theme, registry);
        }
    }
}
```
