use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use crate::core::game_font::tf;
use crate::core::modding::ModRegistry;
use crate::economy::components::PeacefulMode;
use crate::enemy::components::{Enemy, WaveState};
use crate::ui::context::UiDataContext;
use crate::ui::engine::LayoutEngine;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct HudTextComponent;
impl UiComponent for HudTextComponent {
    fn id(&self) -> &str { "hud_text" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, _registry: &ComponentRegistry) -> Entity {
        let data_key = config.get("data_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let font_size = config.get("font_size").and_then(|v| v.as_float()).unwrap_or(14.0) as f32;
        let color_hex = config.get("color").and_then(|v| v.as_str()).unwrap_or("#ffffff");
        let color = parse_hex(color_hex);
        let top = config.get("position").and_then(|v| v.get("top")).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
        let right = config.get("position").and_then(|v| v.get("right")).and_then(|v| v.as_float()).map(|v| v as f32);
        let left = config.get("position").and_then(|v| v.get("left")).and_then(|v| v.as_float()).map(|v| v as f32);
        let bottom = config.get("position").and_then(|v| v.get("bottom")).and_then(|v| v.as_float()).map(|v| v as f32);

        let mut node = Node {
            position_type: PositionType::Absolute,
            top: Val::Px(top),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        };
        if let Some(r) = right { node.right = Val::Px(r); }
        if let Some(l) = left { node.left = Val::Px(l); }
        if let Some(b) = bottom { node.bottom = Val::Px(b); }

        spawn_child(commands, parent, (
            Text::new(String::new()),
            tf(font_size),
            TextColor(color),
            node,
            HudText { data_key },
        ))
    }
}

#[derive(Component, Clone)]
pub struct HudText {
    pub data_key: String,
}

#[derive(Component)]
pub struct HudRoot;

#[derive(Resource)]
pub struct FpsUpdateTimer(pub Timer);

impl Default for FpsUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

/// Spawns the HUD overlay from panel_hud.toml.
pub fn spawn_hud(
    mut commands: Commands,
    mods: Res<ModRegistry>,
    engine: Res<LayoutEngine>,
) {
    let Some(content) = mods.load_data("panel_hud.toml") else { return };
    let Ok(config) = toml::from_str::<toml::Value>(&content) else { return };

    let root = commands.spawn((
        HudRoot,
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO, right: Val::ZERO,
            top: Val::ZERO, bottom: Val::ZERO,
            ..default()
        },
        BackgroundColor(Color::NONE),
        ZIndex(50),
    )).id();

    if let Some(sections) = config.get("sections").and_then(|v| v.as_array()) {
        let dummy = commands.spawn_empty().id();
        let data = UiDataContext::new(dummy, Default::default());
        for section_config in sections {
            let cid = section_config.get("type").and_then(|v| v.as_str()).unwrap_or("hud_text");
            if let Some(comp) = engine.registry.get(cid) {
                comp.render(&mut commands, root, section_config, &data, &engine.theme, &engine.registry);
            }
        }
    }
}

/// Destroys the HUD overlay.
pub fn despawn_hud(mut commands: Commands, root_q: Query<Entity, With<HudRoot>>) {
    use crate::core::utils::silent_despawn;
    for e in &root_q { silent_despawn(&mut commands, e); }
}

/// Updates wave counter text in HUD.
pub fn update_hud_wave_counter(
    wave: Res<WaveState>,
    enemies: Query<Entity, With<Enemy>>,
    peaceful: Res<PeacefulMode>,
    mut q: Query<(&HudText, &mut Text)>,
) {
    for (hud, mut text) in &mut q {
        if hud.data_key != "hud.wave_counter" { continue; }
        text.0 = if peaceful.0 {
            "Peaceful Mode  |  No enemies".to_string()
        } else {
            format!("Wave {}  |  Enemies: {}", wave.wave, enemies.iter().len())
        };
    }
}

/// Updates FPS counter in HUD.
pub fn update_hud_fps(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut timer: ResMut<FpsUpdateTimer>,
    mut q: Query<(&HudText, &mut Text)>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() { return; }
    for (hud, mut text) in &mut q {
        if hud.data_key != "hud.fps" { continue; }
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|d| d.smoothed())
            .map_or("--".to_string(), |v| format!("{:.0}", v));
        text.0 = format!("FPS: {}", fps);
    }
}

fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        Color::srgb(r, g, b)
    } else {
        Color::WHITE
    }
}
