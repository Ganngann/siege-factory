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

    pub fn render_panel(
        &self,
        commands: &mut Commands,
        panel_config: &toml::Value,
        entity: Entity,
        world: &World,
    ) -> (Entity, Entity) {
        let data = UiDataContext::new(entity, world);
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

        if let Some(sections) = panel_config.get("sections").and_then(|v| v.as_array()) {
            for section_config in sections {
                let cid = section_config.get("type").and_then(|v| v.as_str()).unwrap_or("section");
                if let Some(comp) = self.registry.get(cid) {
                    comp.render(commands, root, section_config, &data, &self.theme, &self.registry);
                }
            }
        }

        commands.entity(root).insert(crate::economy::components::PanelModal);
        (overlay, root)
    }
}
