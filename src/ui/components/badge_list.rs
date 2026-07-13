#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::drop_non_drop)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct BadgeListComponent;
impl UiComponent for BadgeListComponent {
    fn id(&self) -> &str { "badge_list" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, data: &UiDataContext, theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let data_key = config.get("data_key").and_then(|v| v.as_str()).unwrap_or("");
        let raw = data.resolve(data_key);

        let container = spawn_child(commands, parent, (
            Node { flex_direction: FlexDirection::Column, width: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::NONE),
        ));

        // Expects TOML array: [[items]]
        // Each item: { id, title, state, separator? }
        if let Ok(val) = toml::from_str::<toml::Value>(&raw)
            && let Some(items) = val.get("items").and_then(|v| v.as_array())
        {
            for item in items {
                    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let state = item.get("state").and_then(|v| v.as_str()).unwrap_or("locked");
                    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let is_sep = item.get("separator").and_then(|v| v.as_bool()).unwrap_or(false);

                    let (badge, bg, tc) = match state {
                        "done" => ("[✓]".to_string(), Color::srgba(0.10, 0.25, 0.10, 1.0), Color::srgb(0.40, 0.80, 0.40)),
                        "current" => (if is_sep { "[!]".into() } else { format!("[{}]", id) }, Color::srgba(0.35, 0.10, 0.10, 1.0), Color::srgb(1.0, 0.40, 0.40)),
                        _ => (if is_sep { "[ ]".into() } else { format!("[{}]", id) }, Color::srgba(0.06, 0.06, 0.12, 1.0), Color::srgb(0.40, 0.40, 0.50)),
                    };

                    commands.entity(container).with_children(|parent| {
                        parent.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                width: Val::Percent(100.0),
                                padding: UiRect::vertical(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                        )).with_children(|row| {
                            row.spawn((Text::new(badge), tf(theme.font_size_small), TextColor(tc)));
                            row.spawn((Text::new(title.to_string()), tf(theme.font_size_small), TextColor(tc)));
                        });
                    });
            }
        }

        container
    }
}
