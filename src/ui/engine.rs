use bevy::prelude::*;
use crate::economy::window::spawn_window;
use crate::ui::context::UiDataContext;
use crate::ui::registry::ComponentRegistry;
use crate::ui::theme::Theme;

#[derive(Resource)]
pub struct LayoutEngine {
    pub registry: ComponentRegistry,
    pub theme: Theme,
}

impl LayoutEngine {
    pub fn new(registry: ComponentRegistry, theme: Theme) -> Self {
        Self { registry, theme }
    }

    /// Renders a building inspection panel: centered window with fullscreen overlay.
    pub fn render_panel(
        &self,
        commands: &mut Commands,
        panel_config: &toml::Value,
        _entity: Entity,
        data: &UiDataContext,
    ) -> (Entity, Entity) {
        let title = panel_config.get("title").and_then(|v| v.as_str()).unwrap_or("Panel");
        let width = panel_config.get("width").and_then(|v| v.as_float()).unwrap_or(800.0) as f32;
        let height = panel_config.get("height").and_then(|v| v.as_float()).unwrap_or(560.0) as f32;
        let x = (1280.0 - width) / 2.0;
        let y = (720.0 - height) / 2.0;

        let overlay = commands.spawn((
            crate::economy::components::PanelOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO, right: Val::ZERO,
                top: Val::ZERO, bottom: Val::ZERO,
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(self.theme.panel_overlay),
            ZIndex(100),
            Pickable::default(),
        )).id();

        let root = spawn_window(commands, title, width, height, x, y, None, |_| {});
        commands.entity(overlay).add_child(root);

        self.render_sections(commands, root, panel_config, data);
        (overlay, root)
    }

    /// Renders a fullscreen overlay (no window frame) with sections as children.
    /// Used for game over, main menu, etc.
    pub fn render_fullscreen(
        &self,
        commands: &mut Commands,
        panel_config: &toml::Value,
        data: &UiDataContext,
    ) -> Entity {
        let bg_color = panel_config
            .get("background")
            .and_then(|v| v.as_str())
            .map(parse_hex)
            .unwrap_or(Color::srgba(0.0, 0.0, 0.0, 0.75));

        let root = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                top: Val::ZERO,
                bottom: Val::ZERO,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            ZIndex(110),
            Pickable::default(),
        )).id();

        self.render_sections(commands, root, panel_config, data);
        root
    }

    /// Renders a persistent HUD/UI element: no overlay, positioned via TOML.
    pub fn render_hud_element(
        &self,
        commands: &mut Commands,
        section_config: &toml::Value,
        data: &UiDataContext,
    ) -> Entity {
        let cid = section_config.get("type").and_then(|v| v.as_str()).unwrap_or("hud_text");
        if let Some(comp) = self.registry.get(cid) {
            // For HUD elements, parent is the root of the UI layer (we spawn at root level)
            let root = commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            )).id();
            comp.render(commands, root, section_config, data, &self.theme, &self.registry);
            root
        } else {
            commands.spawn(Text::new("")).id()
        }
    }

    fn render_sections(
        &self,
        commands: &mut Commands,
        parent: Entity,
        panel_config: &toml::Value,
        data: &UiDataContext,
    ) {
        if let Some(sections) = panel_config.get("sections").and_then(|v| v.as_array()) {
            for section_config in sections {
                let cid = section_config.get("type").and_then(|v| v.as_str()).unwrap_or("section");
                if let Some(comp) = self.registry.get(cid) {
                    comp.render(commands, parent, section_config, data, &self.theme, &self.registry);
                }
            }
        }
    }
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::srgb(r, g, b)
}
