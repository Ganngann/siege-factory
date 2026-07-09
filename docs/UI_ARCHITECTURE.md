# Architecture UI — Siege Factory

> **Statut :** ✅ Framework UI data-driven complet. 11 composants réutilisables, `LayoutEngine`, `ComponentRegistry`, `Theme` data-driven, `UiDataContext`.
> **Dernière mise à jour :** 9 juillet 2026 — Documentation complète dans `docs/UI_COMPONENTS.md`, `docs/UI_PANELS.md`, `docs/UI_COMPONENT_TEMPLATE.md`.

## Problèmes actuels

| Problème | Symptôme |
|----------|----------|
| **Monolithe** | `spawn.rs` = 900+ lignes, un seul fichier pour tous les panneaux |
| **Logique inline** | `spawn_window(content: impl FnOnce(...))` → tout en closures, pas réutilisable |
| **Pas de lifecycle** | open/close/update gérés différemment par chaque panneau |
| **Composants ad-hoc** | 40+ composants marqueurs (`PanelOverlay`, `PanelModal`, etc.) sans hiérarchie claire |
| **Constantes éparpillées** | Couleurs, tailles, polices définies dans `window.rs`, `spawn.rs`, `inspect/mod.rs` |
| **Pas de thème** | Les couleurs sont en dur dans le code Rust |
| **Pas d'event bus** | Chaque système ticke indépendamment → rafraîchissements redondants |
| **Non extensible** | Un mod ne peut pas ajouter son propre panneau UI |

## Architecture cible

```
src/ui/                          # Module UI centralisé
├── mod.rs                       # UiPlugin, UiRegistry, PanelManager
├── styles.rs                    # Thème + constantes de style
├── components.rs                # Bibliothèque de composants réutilisables
├── builder.rs                   # UiBuilder — API fluent
├── event.rs                     # Event Bus centralisé
├── toasts.rs                    # Système de toasts
├── tooltip.rs                   # Tooltips au survol
└── panels/
    ├── mod.rs                   # Panel trait + PanelRegistry
    ├── building.rs              # Panneau inspect bâtiment
    ├── capsule.rs               # Panneau capsule + Data Pad
    ├── deposit.rs               # Panneau dépôt
    ├── crafting.rs              # Panneau craft
    ├── pause.rs                 # Menu pause
    └── main_menu/               # Menu principal
```

---

## ✅ Ce qui a été implémenté

| Fichier | Rôle | Statut |
|---------|------|--------|
| `src/ui/mod.rs` | Module root, déclaration des sous-modules | ✅ |
| `src/ui/styles.rs` | `Theme` struct avec valeurs par défaut + merge TOML | ✅ |
| `src/ui/components.rs` | `PanelType`, `SectionType`, `LabelStyle`, `Panel`, `PanelSection`, `ManagedPanel` | ✅ |
| `src/ui/event.rs` | `PanelEvent`, `CraftEvent`, `ResourceTransferEvent` | ✅ |
| `src/ui/builder.rs` | `UiBuilder` — API fluent : `panel()`, `label()`, `button()`, `progress_bar()`, `icon_text()` | ✅ |
| `src/ui/panels/mod.rs` | `Panel` trait (spawn/update/close) + `PanelRegistry` (extensible par les mods) | ✅ |
| `src/ui/panels/building.rs` | `BuildingPanelImpl` implémente `Panel` (wrapper) | ✅ |
| `src/ui/panels/capsule.rs` | `CapsulePanelImpl` implémente `Panel` (wrapper) | ✅ |
| `src/ui/panels/deposit.rs` | `DepositPanelImpl` implémente `Panel` (wrapper) | ✅ |
| `src/ui/panels/crafting.rs` | `CraftingPanelImpl` implémente `Panel` (wrapper) | ✅ |

### Prochaine étape (Phase 5)

Migrer le contenu de `spawn.rs` (900 lignes) vers les implémentations `Panel` concrètes :
1. Extraire `open_panel` → `ui/panels/building.rs`
2. Extraire `open_capsule_panel` → `ui/panels/capsule.rs`
3. Extraire `spawn_deposit_panel` → `ui/panels/deposit.rs`
4. Supprimer `spawn.rs`, `deposit_panel.rs` et `data_pad_ui.rs`

---

## Phase 1 — Fondations

### Fichiers

| Fichier | Rôle |
|---------|------|
| `src/ui/mod.rs` | Déclaration du module, `UiPlugin`, `PanelManager` ressource |
| `src/ui/styles.rs` | `Theme` struct, constantes de style, chargement TOML |
| `src/ui/components.rs` | Composants de base réutilisables |
| `src/ui/event.rs` | `PanelEvent`, `CraftEvent`, `DiscoveryEvent` |

### `src/ui/components.rs` — Composants de base

```rust
/// Marqueur : une entité est un panneau
#[derive(Component)]
pub struct Panel {
    pub id: String,
    pub panel_type: PanelType,
    pub dirty: bool,
}

pub enum PanelType {
    Building,
    Capsule,
    Deposit,
    Crafting,
    Pause,
    MainMenu,
}

/// Section d'un panneau (permet le ciblage par les update systems)
#[derive(Component)]
pub struct PanelSection {
    pub section: SectionType,
}

pub enum SectionType {
    Header,
    Progression,
    DataPad,
    Inventory,
    Recipes,
    Power,
    Stats,
    Footer,
}

/// Texte stylisé
#[derive(Component)]
pub struct Label {
    pub style: LabelStyle,
}

pub enum LabelStyle {
    Title,
    Body,
    Small,
    Accent,
    Monospace,
}
```

### `src/ui/styles.rs` — Thème

```rust
/// Thème chargé depuis `data/theme.toml` (mergeable entre mods)
pub struct Theme {
    pub panel_bg: Color,
    pub panel_border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub button_bg: Color,
    pub button_hover: Color,
    pub button_active: Color,
    pub font_size_title: f32,
    pub font_size_body: f32,
    pub font_size_small: f32,
}
```

TOML d'exemple :

```toml
[theme]
panel_bg = "#1a1a2e"
panel_border = "#16213e"
text_primary = "#e0e0e0"
text_secondary = "#a0a0a0"
accent = "#0f3460"
success = "#4ecca3"
warning = "#ffc107"
danger = "#e23e57"
button_bg = "#2a2a3e"
button_active = "#3a5a3a"
font_size_title = 16
font_size_body = 12
font_size_small = 10
```

### `src/ui/event.rs` — Event Bus

```rust
/// Événement centralisé pour les actions UI
#[derive(Event)]
pub enum PanelEvent {
    Open { entity: Entity, panel_type: PanelType },
    Close { entity: Entity },
    Update { entity: Entity, section: SectionType },
}

#[derive(Event)]
pub struct CraftEvent {
    pub recipe_id: String,
    pub count: u32,
}

#[derive(Event)]
pub struct DiscoveryEvent {
    pub discovery_id: String,
    pub message: String,
}
```

**Bénéfice** : les systèmes d'UI s'abonnent aux events au lieu de ticker chaque frame :

```rust
// Avant : ticke chaque frame
pub fn update_panel_inventory(panel: Res<BuildingPanel>, ...) { ... }

// Après : ne ticke que quand nécessaire
pub fn on_panel_update(event: On<PanelEvent>, ...) {
    if let PanelEvent::Update { section: Inventory, .. } = event.event() { ... }
}
```

---

## Phase 2 — UiBuilder (API fluent)

### `src/ui/builder.rs`

```rust
pub struct UiBuilder<'w, 's> {
    commands: &'w mut Commands<'s>,
    theme: &'w Theme,
}

impl UiBuilder {
    /// Crée un panneau modale avec overlay
    pub fn panel(
        &mut self, title: &str, w: f32, h: f32, x: f32, y: f32
    ) -> PanelBuilder;

    /// Texte stylisé
    pub fn label(&mut self, text: &str, style: LabelStyle) -> &mut Self;

    /// Bouton cliquable
    pub fn button(&mut self, text: &str) -> ButtonBuilder;

    /// Barre de progression
    pub fn progress_bar(&mut self, current: u32, max: u32) -> &mut Self;

    /// Liste défilante
    pub fn scroll_list<T>(
        &mut self, items: &[T],
        template: impl Fn(&mut Self, &T)
    ) -> &mut Self;

    /// Texte avec icône (● / ○)
    pub fn icon_text(&mut self, icon: &str, text: &str, style: LabelStyle) -> &mut Self;
}

pub struct PanelBuilder<'w, 's, 'a> { ... }

impl PanelBuilder {
    pub fn section(&mut self, header: &str) -> SectionBuilder;
    pub fn build(self) -> Entity;
}

pub struct SectionBuilder { ... }

impl SectionBuilder {
    pub fn label(...) -> &mut Self;
    pub fn button(...) -> &mut Self;
    pub fn progress_bar(...) -> &mut Self;
    pub fn scroll_list(...) -> &mut Self;
}
```

### Exemple d'utilisation

```rust
// Code actuel (~80 lignes)
fn open_panel(commands, ..., panel, entity, ...) {
    let overlay = commands.spawn((...)).id();
    let root = spawn_window(&mut commands, ..., |parent| {
        parent.spawn((...));
        parent.spawn((...));
    });
    commands.entity(overlay).add_child(root);
    panel.root = Some(root);
    panel.overlay = Some(overlay);
}

// Code cible (~15 lignes)
fn open_panel(ui: &mut UiBuilder) {
    ui.panel("Four", 800.0, 560.0, 120.0, 80.0)
        .section("Progression")
            .label("Tier 2/7", LabelStyle::Title)
            .progress_bar(2, 7)
        .section("Data Pad")
            .scroll_list(&logs, |ui, log| {
                ui.button(&log.title);
            })
        .build();
}
```

---

## Phase 3 — Panel trait + registre

### `src/ui/panels/mod.rs`

```rust
/// Un panneau UI standardisé
pub trait Panel: Send + Sync {
    /// Identifiant unique du type de panneau (ex: "building", "capsule")
    fn panel_type(&self) -> &str;

    /// Crée l'entité panneau et retourne son ID
    fn spawn(
        &self,
        commands: &mut Commands,
        panel: &mut BuildingPanel,
        entity: Entity,
    );

    /// Met à jour le contenu du panneau (quand dirty)
    fn update(
        &self,
        commands: &mut Commands,
        panel: &BuildingPanel,
    );

    /// Nettoie l'entité panneau
    fn close(
        &self,
        commands: &mut Commands,
        panel: &mut BuildingPanel,
    );
}

/// Registre des types de panneaux accessibles par les mods
pub struct PanelRegistry {
    panels: HashMap<String, Box<dyn Panel>>,
}

impl PanelRegistry {
    pub fn register(&mut self, panel: Box<dyn Panel>);
    pub fn get(&self, panel_type: &str) -> Option<&dyn Panel>;
}
```

### `PanelManager` — ressource centralisée

```rust
#[derive(Resource)]
pub struct PanelManager {
    pub active_panels: Vec<ActivePanel>,
    pub registry: PanelRegistry,
}

pub struct ActivePanel {
    pub entity: Entity,
    pub panel_type: String,
    pub root: Option<Entity>,
    pub overlay: Option<Entity>,
}
```

### Extensibilité mods

Un mod enregistre son panneau au démarrage :

```rust
impl Plugin for MyModPlugin {
    fn build(&self, app: &mut App) {
        let registry = app.world().resource_mut::<PanelManager>();
        registry.register(Box::new(MyCustomPanel));
    }
}
```

---

## Phase 4 — Migration des panneaux existants

### `building.rs` (remplace spawn.rs building)

```rust
pub struct BuildingPanelImpl;

impl Panel for BuildingPanelImpl {
    fn panel_type(&self) -> &str { "building" }

    fn spawn(&self, commands: &mut Commands, panel: &mut BuildingPanel, entity: Entity) {
        let def = registry.get(&building.kind);
        let mut ui = UiBuilder::new(commands, &theme);

        ui.panel(&building.name, 800.0, 560.0, 120.0, 80.0)
            .section("Status")
                .label("Actif", LabelStyle::Body)
            .section("Production")
                .label(infos.production, LabelStyle::Body)
                .progress_bar(infos.progress, 100)
            .section("Inventaire")
                .scroll_list(&inventory, |ui, slot| {
                    ui.label(&slot.resource, LabelStyle::Body);
                })
            .build();
    }
}
```

### `capsule.rs` (remplace spawn.rs capsule)

```rust
impl Panel for CapsulePanelImpl {
    fn spawn(&self, commands, panel, entity) {
        let mut ui = UiBuilder::new(commands, &theme);

        ui.panel("Capsule Genesis", 520.0, 320.0, 120.0, 80.0)
            .section("Progression")
                .label("Tiers: 3/7", LabelStyle::Title)
                .label("✅ Tier 0 — Déblayage (fait)", LabelStyle::Body)
                .label("◉ Tier 1 — Réveil (en cours)", LabelStyle::Accent)
                .label("○ Tier 2 — Étincelle", LabelStyle::Small)
            .section("Items requis")
                .label("Pièces de Fer  0/5", LabelStyle::Body)
            .section("Data Pad")
                .scroll_list(&logs, |ui, entry| {
                    ui.button(&format!("●  {}", entry.title));
                })
            .build();
    }
}
```

---

## Phase 5 — Nettoyage

### Fichiers supprimés

| Fichier | Remplacé par |
|---------|-------------|
| `src/economy/inspect/spawn.rs` | `src/ui/panels/building.rs` + `capsule.rs` |
| `src/economy/inspect/deposit_panel.rs` | `src/ui/panels/deposit.rs` |
| `src/economy/inspect/recipe_selector.rs` | Intégré dans `building.rs` |
| `src/economy/inspect/data_pad_ui.rs` | Intégré dans `capsule.rs` |
| `src/economy/inspect/update.rs` | `Panel::update()` |
| `src/economy/inspect/interaction.rs` (partiel) | `PanelManager.active_panels` |
| `src/economy/window.rs` (partiel) | `UiBuilder::panel()` |
| `src/player/crafting.rs` (partiel) | `src/ui/panels/crafting.rs` |
| `src/core/toast.rs` | `src/ui/toasts.rs` |
| `src/rendering/building_tooltip.rs` | `src/ui/tooltip.rs` |

### Fichiers simplifiés

| Fichier | Changement |
|---------|------------|
| `src/economy/inspect/mod.rs` | Plus que la redirection vers `PanelManager` |
| `src/economy/mod.rs` | Plus de registrations inspect → remplacées par `UiPlugin` |
| `src/core/mod.rs` | Plus de `pub mod toast` |
| `src/player/mod.rs` | Plus de `pub mod crafting` (UI) |

---

## Phase 6 — Thème TOML (optionnel)

```toml
# mods/genesis_protocol/data/theme.toml
[theme]
panel_bg = "#1a1a2e"
panel_border = "#16213e"
text_primary = "#e0e0e0"
text_secondary = "#a0a0a0"
accent = "#0f3460"
success = "#4ecca3"
warning = "#ffc107"
danger = "#e23e57"
font_size_title = 16
font_size_body = 12
font_size_small = 10
```

Le thème est mergeable entre mods (comme `map_config.toml`) :

```rust
impl Theme {
    pub fn load(mods: &ModRegistry) -> Self {
        let mut theme = Self::default();
        for (_mod_id, parsed) in mods.load_all_toml::<ThemeToml>("theme.toml") {
            theme.merge(parsed.theme);
        }
        theme
    }
}
```

---

## Résumé des phases

| Phase | Fichiers créés | Fichiers supprimés | Δ lignes |
|-------|---------------|-------------------|----------|
| **1** Fondations ✅ | `ui/mod.rs`, `styles.rs`, `components.rs`, `event.rs` | — | +200 |
| **2** Builder ✅ | `ui/builder.rs` | — | +250 |
| **3** Panel trait ✅ | `ui/panels/mod.rs` | — | +150 |
| **4** Wrappers ✅ | `ui/panels/building.rs`, `capsule.rs`, `deposit.rs`, `crafting.rs` | — (coexistence) | +40 |
| **5** Migration contenu ⏳ | `spawn.rs` → `ui/panels/` | `spawn.rs` (quand migré) | −700 |
| **6** Nettoyage | `ui/toasts.rs`, `ui/tooltip.rs` | `toast.rs` (déplacé), `building_tooltip.rs` (déplacé) | −100 |
| **7** Thème TOML | — | — | +80 |
| **Total** | 11 fichiers | 7 fichiers | **+280 lignes nettes** |

L'architecture passe de monolithe (1 fichier de 900 lignes) à modulaire (11 fichiers spécialisés), avec une API propre et extensible par les mods.

---

## Rétrocompatibilité

Chaque phase est conçue pour ne **rien casser** :

- **Phase 1** : ne fait qu'ajouter des définitions. Rien ne change.
- **Phase 2** : `UiBuilder` cohabite avec `spawn_window`. Migration progressive.
- **Phase 3** : `PanelRegistry` existe en parallèle du `BuildingPanel` actuel.
- **Phase 4** : les anciens fichiers sont supprimés seulement après que TOUS les appels sont migrés.
- **Tous les tests passent à chaque étape** (432 tests).
