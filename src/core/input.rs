use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

#[derive(Resource, Debug, Clone)]
pub struct KeyBindings {
    map: HashMap<String, InputBinding>,
}

impl KeyBindings {
    pub fn load() -> Self {
        let raw: BindingsToml = toml::from_str(include_str!("../../data/keybindings.toml"))
            .expect("failed to parse data/keybindings.toml");
        let mut map = HashMap::new();
        for (action, name) in raw.bindings {
            let binding = parse_input_binding(&name)
                .unwrap_or_else(|| panic!("unknown binding name '{name}' for action '{action}'"));
            map.insert(action, binding);
        }
        Self { map }
    }

    pub fn get(&self, action: &str) -> InputBinding {
        self.map.get(action).copied().unwrap_or_else(|| {
            panic!("action '{action}' not found in keybindings")
        })
    }

    pub fn key(&self, action: &str) -> KeyCode {
        match self.get(action) {
            InputBinding::Key(k) => k,
            InputBinding::Mouse(_) => panic!("action '{action}' is a mouse binding, not a key"),
        }
    }

    pub fn mouse(&self, action: &str) -> MouseButton {
        match self.get(action) {
            InputBinding::Mouse(m) => m,
            InputBinding::Key(_) => panic!("action '{action}' is a key binding, not a mouse"),
        }
    }

    pub fn just_pressed(
        &self,
        action: &str,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self.get(action) {
            InputBinding::Key(k) => keys.just_pressed(k),
            InputBinding::Mouse(m) => mouse.just_pressed(m),
        }
    }

    pub fn just_released(&self, action: &str, mouse: &ButtonInput<MouseButton>) -> bool {
        match self.get(action) {
            InputBinding::Mouse(m) => mouse.just_released(m),
            InputBinding::Key(_) => false,
        }
    }

    pub fn all(&self) -> Vec<(String, InputBinding)> {
        let mut pairs: Vec<_> = self.map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    }

    pub fn set(&mut self, action: &str, binding: InputBinding) {
        self.map.insert(action.to_string(), binding);
    }

    pub fn apply_overrides(&mut self, overrides: &std::collections::HashMap<String, String>) {
        for (action, name) in overrides {
            if let Some(binding) = parse_input_binding(name) {
                self.map.insert(action.clone(), binding);
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct BindingsToml {
    bindings: HashMap<String, String>,
}

fn parse_input_binding(name: &str) -> Option<InputBinding> {
    if let Some(key) = parse_key_code(name) {
        return Some(InputBinding::Key(key));
    }
    if let Some(mouse) = parse_mouse_button(name) {
        return Some(InputBinding::Mouse(mouse));
    }
    None
}

fn parse_mouse_button(name: &str) -> Option<MouseButton> {
    Some(match name {
        "MouseLeft" => MouseButton::Left,
        "MouseRight" => MouseButton::Right,
        "MouseMiddle" => MouseButton::Middle,
        _ => return None,
    })
}

fn parse_key_code(name: &str) -> Option<KeyCode> {
    Some(match name {
        "Space" => KeyCode::Space,
        "Escape" => KeyCode::Escape,
        "Enter" => KeyCode::Enter,
        "Tab" => KeyCode::Tab,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,

        n if n.starts_with("Key") && n.len() == 4 => {
            let c = n.as_bytes()[3];
            if c.is_ascii_uppercase() {
                match c {
                    b'A' => KeyCode::KeyA,
                    b'B' => KeyCode::KeyB,
                    b'C' => KeyCode::KeyC,
                    b'D' => KeyCode::KeyD,
                    b'E' => KeyCode::KeyE,
                    b'F' => KeyCode::KeyF,
                    b'G' => KeyCode::KeyG,
                    b'H' => KeyCode::KeyH,
                    b'I' => KeyCode::KeyI,
                    b'J' => KeyCode::KeyJ,
                    b'K' => KeyCode::KeyK,
                    b'L' => KeyCode::KeyL,
                    b'M' => KeyCode::KeyM,
                    b'N' => KeyCode::KeyN,
                    b'O' => KeyCode::KeyO,
                    b'P' => KeyCode::KeyP,
                    b'Q' => KeyCode::KeyQ,
                    b'R' => KeyCode::KeyR,
                    b'S' => KeyCode::KeyS,
                    b'T' => KeyCode::KeyT,
                    b'U' => KeyCode::KeyU,
                    b'V' => KeyCode::KeyV,
                    b'W' => KeyCode::KeyW,
                    b'X' => KeyCode::KeyX,
                    b'Y' => KeyCode::KeyY,
                    b'Z' => KeyCode::KeyZ,
                    _ => return None,
                }
            } else {
                return None;
            }
        }

        n if n.starts_with("Digit") && n.len() == 6 => {
            let d = n.as_bytes()[5];
            match d {
                b'0' => KeyCode::Digit0,
                b'1' => KeyCode::Digit1,
                b'2' => KeyCode::Digit2,
                b'3' => KeyCode::Digit3,
                b'4' => KeyCode::Digit4,
                b'5' => KeyCode::Digit5,
                b'6' => KeyCode::Digit6,
                b'7' => KeyCode::Digit7,
                b'8' => KeyCode::Digit8,
                b'9' => KeyCode::Digit9,
                _ => return None,
            }
        }

        n if n.starts_with('F') && n.len() <= 3 => {
            let num: u8 = n[1..].parse().ok()?;
            match num {
                1 => KeyCode::F1,
                2 => KeyCode::F2,
                3 => KeyCode::F3,
                4 => KeyCode::F4,
                5 => KeyCode::F5,
                6 => KeyCode::F6,
                7 => KeyCode::F7,
                8 => KeyCode::F8,
                9 => KeyCode::F9,
                10 => KeyCode::F10,
                11 => KeyCode::F11,
                12 => KeyCode::F12,
                _ => return None,
            }
        }

        _ => return None,
    })
}
