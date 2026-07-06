use crate::load_toml;
use super::types::*;

impl MainMenuDef {
    pub fn load() -> Self {
        let raw: MenuToml = load_toml!("../../../data/main_menu.toml", MenuToml);

        let screens = raw
            .screen
            .into_iter()
            .map(|(id, ts)| {
                let items = ts
                    .items
                    .into_iter()
                    .filter_map(|ti| {
                        let action = match ti.action.as_str() {
                            "StartGame" => MenuAction::StartGame,
                            "StartPeaceful" => MenuAction::StartPeaceful,
                            "OpenScreen" => {
                                MenuAction::OpenScreen(ti.target.clone().unwrap_or_default())
                            }
                            "Back" => MenuAction::Back,
                            "Quit" => MenuAction::Quit,
                            "LoadGame" => MenuAction::LoadGame,
                            _ => return None,
                        };
                        Some(MenuItemDef {
                            id: ti.id,
                            label: ti.label,
                            action,
                        })
                    })
                    .collect();
                (
                    id,
                    ScreenDef {
                        title: ts.title,
                        subtitle: ts.subtitle,
                        items,
                    },
                )
            })
            .collect();

        Self { screens }
    }
}
