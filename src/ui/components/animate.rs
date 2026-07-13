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
use crate::ui::context::UiDataContext;
use crate::ui::registry::{UiComponent, spawn_child};
use crate::ui::theme::Theme;
use crate::ui::registry::ComponentRegistry;

pub struct AnimateComponent;
impl UiComponent for AnimateComponent {
    fn id(&self) -> &str { "animate" }
    fn render(&self, commands: &mut Commands, parent: Entity, config: &toml::Value, _data: &UiDataContext, _theme: &Theme, registry: &ComponentRegistry) -> Entity {
        let effect = config.get("effect").and_then(|v| v.as_str()).unwrap_or("none").to_string();
        let duration = config.get("duration").and_then(|v| v.as_float()).unwrap_or(1.0);
        let children = config.get("children").and_then(|v| v.as_array());

        let container = spawn_child(commands, parent, (
            Node::default(),
            BackgroundColor(Color::NONE),
            AnimationState { effect, timer: 0.0, duration: duration as f32, phase: 0.0 },
        ));

        if let Some(arr) = children {
            for child_config in arr {
                let cid = child_config.get("type").and_then(|v| v.as_str()).unwrap_or("label");
                if let Some(comp) = registry.get(cid) {
                    comp.render(commands, container, child_config, _data, _theme, registry);
                }
            }
        }

        container
    }
}

#[derive(Component, Clone)]
pub struct AnimationState {
    pub effect: String,
    pub timer: f32,
    pub duration: f32,
    pub phase: f32,
}

pub fn animation_tick_system(
    time: Res<Time>,
    mut q: Query<(&mut AnimationState, &mut BackgroundColor)>,
) {
    for (mut state, mut bg) in q.iter_mut() {
        state.timer += time.delta_secs();
        if state.timer > state.duration {
            state.timer = 0.0;
        }
        let progress = state.timer / state.duration;
        state.phase = match state.effect.as_str() {
            "pulse" => (progress * std::f32::consts::TAU).sin() * 0.35 + 0.65,
            "blink" => if progress < 0.5 { 1.0 } else { 0.0 },
            "flicker" => {
                let r: f32 = (state.timer * 10.0).fract();
                if r < 0.05 { 0.2 } else if r < 0.08 { 0.8 } else if r < 0.11 { 0.3 } else { 0.9 + (r * 0.1) }
            },
            _ => 1.0,
        };
        let srgb = bg.0.to_srgba();
        let alpha = (srgb.alpha as f32 * state.phase).min(1.0);
        bg.0 = Color::srgba(srgb.red as f32, srgb.green as f32, srgb.blue as f32, alpha);
    }
}
