use bevy::prelude::*;
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;
use crate::rendering::minimap::MinimapCamera;

#[derive(Component)]
pub struct MinimapBorderConfig {
    pub size: f32,
    pub margin: f32,
    pub border_width: f32,
    pub border_color: Color,
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return Color::srgb(0.2, 1.0, 0.2); }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::srgb(r, g, b)
}

pub struct MinimapComponent;
impl UiComponent for MinimapComponent {
    fn id(&self) -> &str { "minimap" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let size = config.get("size").and_then(|v| v.as_float()).unwrap_or(200.0) as f32;
        let margin = config.get("margin").and_then(|v| v.as_float()).unwrap_or(10.0) as f32;
        let border_width = config.get("border").and_then(|b| b.get("width")).and_then(|v| v.as_float()).unwrap_or(1.0) as f32;
        let border_color_hex = config.get("border").and_then(|b| b.get("color")).and_then(|v| v.as_str()).unwrap_or("#33ff33");
        let border_color = parse_hex(border_color_hex);

        let border_cfg = MinimapBorderConfig { size, margin, border_width, border_color };

        let root = spawn_child(commands, parent, MinimapRoot);

        commands.entity(root).with_children(|p| {
            p.spawn((
                Camera2d,
                MinimapCamera,
                border_cfg,
                Camera {
                    order: 1,
                    viewport: Some(bevy::camera::Viewport {
                        physical_position: UVec2::ZERO,
                        physical_size: UVec2::new(size as u32, size as u32),
                        ..default()
                    }),
                    ..default()
                },
                Projection::Orthographic(OrthographicProjection {
                    scale: 10.0,
                    ..OrthographicProjection::default_2d()
                }),
            ));

            // Border sprites as children of camera (in camera local space)
            let s = size / 10.0;
            let bw = border_width / 10.0;
            let half = s / 2.0;

            // Top
            p.spawn((
                Sprite::from_color(border_color, Vec2::new(s, bw)),
                Transform::from_xyz(0.0, half - bw / 2.0, 0.0),
                MinimapBorder,
            ));
            // Bottom
            p.spawn((
                Sprite::from_color(border_color, Vec2::new(s, bw)),
                Transform::from_xyz(0.0, -half + bw / 2.0, 0.0),
                MinimapBorder,
            ));
            // Left
            p.spawn((
                Sprite::from_color(border_color, Vec2::new(bw, s)),
                Transform::from_xyz(-half + bw / 2.0, 0.0, 0.0),
                MinimapBorder,
            ));
            // Right
            p.spawn((
                Sprite::from_color(border_color, Vec2::new(bw, s)),
                Transform::from_xyz(half - bw / 2.0, 0.0, 0.0),
                MinimapBorder,
            ));
        });

        root
    }
}

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component)]
pub struct MinimapBorder;

pub fn update_minimap_border_system(
    windows: Query<&Window>,
    mut minimap_q: Query<(&mut Camera, &MinimapBorderConfig, &Children), With<MinimapCamera>>,
    mut border_q: Query<&mut Transform, With<MinimapBorder>>,
) {
    let Ok(window) = windows.single() else { return; };
    for (mut camera, cfg, children) in minimap_q.iter_mut() {
        let size_u = UVec2::new(cfg.size as u32, cfg.size as u32);
        let pos = UVec2::new(
            window.resolution.physical_width().saturating_sub(size_u.x + cfg.margin as u32),
            window.resolution.physical_height().saturating_sub(size_u.y + cfg.margin as u32),
        );
        camera.viewport = Some(bevy::camera::Viewport {
            physical_position: pos,
            physical_size: size_u,
            ..default()
        });

        for child in children.iter() {
            if let Ok(mut tf) = border_q.get_mut(child) {
                let half = cfg.size / 10.0 / 2.0;
                let bw = cfg.border_width / 10.0;
                if tf.translation.y.abs() > tf.translation.x.abs() {
                    let sign = if tf.translation.y > 0.0 { 1.0 } else { -1.0 };
                    tf.translation.y = sign * (half - bw / 2.0);
                } else {
                    let sign = if tf.translation.x > 0.0 { 1.0 } else { -1.0 };
                    tf.translation.x = sign * (half - bw / 2.0);
                }
            }
        }
    }
}
