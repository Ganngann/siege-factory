use bevy::prelude::*;

use crate::core::game_font::tf;
use crate::core::modding::ModRegistry;
use crate::core::toast::{ToastMessage, ToastQueue};
use crate::core::utils::silent_despawn;

// ── Config ──

#[derive(Resource)]
pub struct ToastConfig {
    pub bottom_px: f32,
    pub font_size: f32,
    pub color: Color,
    pub lifetime: f32,
    pub max_width: f32,
    pub container: Option<Entity>,
}

impl ToastConfig {
    pub fn load(mods: &ModRegistry) -> Self {
        let content = mods.load_data("panel_toast.toml").unwrap_or_default();
        let Ok(config) = toml::from_str::<toml::Value>(&content) else {
            return Self::default();
        };
        Self {
            bottom_px: config.get("position").and_then(|p| p.get("bottom")).and_then(|v| v.as_float()).unwrap_or(90.0) as f32,
            font_size: config.get("font_size").and_then(|v| v.as_float()).unwrap_or(16.0) as f32,
            color: config.get("color").and_then(|v| v.as_str()).map(parse_hex).unwrap_or(Color::srgb(1.0, 0.85, 0.33)),
            lifetime: config.get("lifetime").and_then(|v| v.as_float()).unwrap_or(5.0) as f32,
            max_width: config.get("max_width").and_then(|v| v.as_float()).unwrap_or(500.0) as f32,
            container: None,
        }
    }
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            bottom_px: 90.0,
            font_size: 16.0,
            color: Color::srgb(1.0, 0.85, 0.33),
            lifetime: 5.0,
            max_width: 500.0,
            container: None,
        }
    }
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return Color::srgb(0.5, 0.5, 0.5); }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    Color::srgb(r, g, b)
}

// ── Container ──

#[derive(Component)]
pub struct ToastContainer;

pub fn spawn_toast_container(mut commands: Commands, mut config: ResMut<ToastConfig>) {
    let container = commands
        .spawn((
            ToastContainer,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Auto,
                right: Val::Auto,
                bottom: Val::Px(config.bottom_px),
                justify_content: JustifyContent::Center,
                max_width: Val::Px(config.max_width),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();
    config.container = Some(container);
}

pub fn despawn_toast_container(
    mut commands: Commands,
    q: Query<Entity, With<ToastContainer>>,
    mut config: ResMut<ToastConfig>,
) {
    for e in &q {
        silent_despawn(&mut commands, e);
    }
    config.container = None;
}

// ── Toast system ──

pub fn toast_system(
    mut commands: Commands,
    mut queue: ResMut<ToastQueue>,
    time: Res<Time>,
    mut toasts: Query<(Entity, &mut ToastMessage)>,
    config: Res<ToastConfig>,
) {
    for msg in queue.0.drain(..) {
        let persistent = msg.starts_with("\x00PERSISTENT\x00");
        let text = if persistent { &msg[12..] } else { &msg };

        let entity = commands.spawn((
            ToastMessage {
                timer: config.lifetime,
                persistent,
            },
            Text::new(text.to_string()),
            tf(config.font_size),
            TextColor(config.color),
            TextLayout::new(Justify::Center, bevy::text::LineBreak::WordBoundary),
            Node {
                max_width: Val::Px(config.max_width),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
        )).id();

        if let Some(container) = config.container {
            commands.entity(container).add_child(entity);
        }
    }

    for (entity, mut msg) in toasts.iter_mut() {
        if msg.persistent {
            continue;
        }
        msg.timer -= time.delta_secs();
        if msg.timer <= 0.0 {
            silent_despawn(&mut commands, entity);
        }
    }
}
