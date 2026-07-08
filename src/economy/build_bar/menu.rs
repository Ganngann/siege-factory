use crate::economy::components::{
    BackButton, BreadcrumbText, MenuBarPanel, MenuItemButton, ScrollButton,
};
use crate::economy::menu::MenuItems;
use crate::rendering::TextureCache;
use bevy::prelude::*;

use super::{
    BACK_BUTTON_WIDTH, BORDER_WIDTH, ITEM_HEIGHT, ITEM_WIDTH, PANEL_HEIGHT, SCROLL_BUTTON_WIDTH,
};

fn slot_key(index: usize) -> &'static str {
    match index {
        0 => "2",
        1 => "3",
        2 => "4",
        3 => "5",
        4 => "6",
        5 => "7",
        6 => "8",
        7 => "9",
        _ => "0",
    }
}

pub fn build_menu_bar(commands: &mut Commands, menu_items: &MenuItems, textures: &TextureCache) {
    commands
        .spawn((
            MenuBarPanel,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(PANEL_HEIGHT),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.85)),
            Pickable::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                BreadcrumbText,
                Text::new(&menu_items.breadcrumb),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgba(0.8, 0.8, 0.9, 0.8)),
                Node {
                    height: Val::Px(16.0),
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    justify_content: JustifyContent::FlexStart,
                    width: Val::Percent(100.0),
                    ..default()
                })
                .with_children(|row| {
                    if menu_items.has_back {
                        row.spawn((
                            BackButton,
                            Button,
                            Node {
                                width: Val::Px(BACK_BUTTON_WIDTH),
                                height: Val::Px(ITEM_HEIGHT),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(BORDER_WIDTH)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("<-1 Retour"),
                                TextFont::from_font_size(11.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                    } else {
                        row.spawn((
                            Node {
                                width: Val::Px(BACK_BUTTON_WIDTH),
                                height: Val::Px(ITEM_HEIGHT),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(BORDER_WIDTH)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.25, 0.5)),
                            BorderColor::all(Color::srgba(0.3, 0.3, 0.3, 0.3)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("1"),
                                TextFont::from_font_size(11.0),
                                TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
                            ));
                        });
                    }

                    if menu_items.can_scroll_left {
                        row.spawn((
                            ScrollButton(-1),
                            Button,
                            Node {
                                width: Val::Px(SCROLL_BUTTON_WIDTH),
                                height: Val::Px(ITEM_HEIGHT),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("<"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                    } else {
                        row.spawn(Node {
                            width: Val::Px(SCROLL_BUTTON_WIDTH),
                            ..default()
                        });
                    }

                    for (i, item) in menu_items.items.iter().enumerate() {
                        let key = slot_key(i);
                        let sub_prefix = match &item.kind {
                            crate::economy::menu::FlatItemKind::SubMenu => "› ",
                            _ => "",
                        };
                        let bg_color = item.color;

                        row.spawn((
                            MenuItemButton { index: i },
                            Button,
                            Node {
                                width: Val::Px(ITEM_WIDTH),
                                height: Val::Px(ITEM_HEIGHT),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(BORDER_WIDTH)),
                                ..default()
                            },
                            BackgroundColor(bg_color),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                        ))
                        .with_children(|b| {
                            if let Some(stem) = &item.texture_stem
                                && let Some(handle) = textures.base.get(stem) {
                                    b.spawn((
                                        ImageNode::new(handle.clone()),
                                        Node {
                                            width: Val::Px(32.0),
                                            height: Val::Px(32.0),
                                            ..default()
                                        },
                                    ));
                                }
                            b.spawn((
                                Text::new(format!("{} {}", key, sub_prefix)),
                                TextFont::from_font_size(9.0),
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                            ));
                            b.spawn((
                                Text::new(&item.label),
                                TextFont::from_font_size(12.0),
                                TextColor(Color::WHITE),
                            ));
                            if !item.cost_str.is_empty() {
                                b.spawn((
                                    Text::new(&item.cost_str),
                                    TextFont::from_font_size(9.0),
                                    TextColor(Color::srgb(1.0, 0.85, 0.3)),
                                ));
                            }
                        });
                    }

                    if menu_items.can_scroll_right {
                        row.spawn((
                            ScrollButton(1),
                            Button,
                            Node {
                                width: Val::Px(SCROLL_BUTTON_WIDTH),
                                height: Val::Px(ITEM_HEIGHT),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(">"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                    } else {
                        row.spawn(Node {
                            width: Val::Px(SCROLL_BUTTON_WIDTH),
                            ..default()
                        });
                    }
                });
        });
}
