use crate::combat::Projectile;
use crate::core::game_state::GameState;
use crate::economy::belt::BeltSlots;
use crate::economy::building::BuildingRegistry;
use crate::economy::components::{
    BuildMode, Building, Direction, HasHpBar, HpBarChild, PeacefulMode, Unit,
};
use crate::economy::resource::ResourceId;
use crate::economy::unit_config::UnitConfig;
use crate::enemy::components::{Enemy, GameOverUi, Health, WaveCounterText, WaveState};
use crate::enemy::registry::EnemyRegistry;
use crate::events::SpawnProjectileEvent;
use crate::map::components::HoveredTile;
use crate::map::config::MapConfig;
use crate::unit::{Soldier, Worker};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use std::collections::HashMap;

// ── Shape cache (Mesh2d fallback for ghosts, projectiles, etc.) ──

#[derive(Resource)]
pub struct ShapeCache {
    pub square: Handle<Mesh>,
    pub diamond: Handle<Mesh>,
    pub triangle: Handle<Mesh>,
    pub rectangle: Handle<Mesh>,
    pub pentagon: Handle<Mesh>,
    pub circle: Handle<Mesh>,
}

impl FromWorld for ShapeCache {
    fn from_world(world: &mut World) -> Self {
        let s = {
            let cfg = world.resource::<MapConfig>();
            cfg.tile_size - 4.0
        };
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self {
            square: meshes.add(Rectangle::new(s, s)),
            diamond: meshes.add(RegularPolygon::new(s * 0.45, 4)),
            triangle: meshes.add(Triangle2d::new(
                Vec2::new(0.0, s * 0.4),
                Vec2::new(-s * 0.4, -s * 0.4),
                Vec2::new(s * 0.4, -s * 0.4),
            )),
            rectangle: meshes.add(Rectangle::new(s * 0.7, s * 0.35)),
            pentagon: meshes.add(RegularPolygon::new(s * 0.4, 5)),
            circle: meshes.add(Circle::new(s * 0.4)),
        }
    }
}

impl ShapeCache {
    pub fn get_visual(&self, visual: &str) -> Handle<Mesh> {
        match visual {
            "square" => self.square.clone(),
            "diamond" => self.diamond.clone(),
            "triangle" => self.triangle.clone(),
            "rectangle" => self.rectangle.clone(),
            "pentagon" => self.pentagon.clone(),
            "circle" => self.circle.clone(),
            _ => self.square.clone(),
        }
    }
}

// ── Preview materials cache (shared handles, no per-frame allocation) ──

#[derive(Resource)]
pub struct PreviewMaterials {
    pub deconstruct_building: Handle<ColorMaterial>,
    pub deconstruct_zone: Handle<ColorMaterial>,
    pub build_valid: Handle<ColorMaterial>,
    pub build_invalid: Handle<ColorMaterial>,
    pub indicator_input: Handle<ColorMaterial>,
    pub indicator_output: Handle<ColorMaterial>,
}

impl FromWorld for PreviewMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        Self {
            deconstruct_building: materials.add(Color::srgba(0.8, 0.0, 0.0, 0.45)),
            deconstruct_zone: materials.add(Color::srgba(0.8, 0.0, 0.0, 0.12)),
            build_valid: materials.add(Color::srgba(0.0, 0.8, 0.0, 0.4)),
            build_invalid: materials.add(Color::srgba(0.8, 0.0, 0.0, 0.3)),
            indicator_input: materials.add(Color::srgba(0.0, 1.0, 0.0, 0.7)),
            indicator_output: materials.add(Color::srgba(0.3, 0.6, 1.0, 0.7)),
        }
    }
}

// ── Texture cache (Sprite-based rendering for buildings/units) ──

#[derive(Resource, Default)]
pub struct TextureCache {
    pub base: HashMap<String, Handle<Image>>,
    pub owner: HashMap<String, Option<Handle<Image>>>,
    pub level: HashMap<String, Option<Handle<Image>>>,
}

impl TextureCache {
    pub fn base(&self, stem: &str) -> Handle<Image> {
        self.base.get(stem).cloned().unwrap_or_default()
    }
    pub fn owner(&self, stem: &str) -> Option<Handle<Image>> {
        self.owner.get(stem).and_then(|h| h.clone())
    }
    pub fn level(&self, stem: &str) -> Option<Handle<Image>> {
        self.level.get(stem).and_then(|h| h.clone())
    }
}

fn setup_texture_cache(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let building_stems = all_texture_stems();
    let item_stems = all_item_stems();
    let total = building_stems.len() + item_stems.len();
    let mut base = HashMap::with_capacity(total);
    let mut owner = HashMap::with_capacity(total);
    let mut level = HashMap::with_capacity(total);

    for stem in &building_stems {
        let s = stem.as_str();
        base.insert(
            stem.clone(),
            load_png(&mut images, s, "base").unwrap_or_default(),
        );
        owner.insert(stem.clone(), load_png(&mut images, s, "owner"));
        level.insert(stem.clone(), load_png(&mut images, s, "level"));
    }

    for stem in &item_stems {
        let s = stem.as_str();
        base.insert(
            stem.clone(),
            load_png(&mut images, s, "base").unwrap_or_default(),
        );
    }

    commands.insert_resource(TextureCache { base, owner, level });
}

fn load_png(images: &mut Assets<Image>, stem: &str, layer: &str) -> Option<Handle<Image>> {
    let path = format!("assets/textures/{}_{}.png", stem, layer);
    let data = std::fs::read(&path).ok()?;
    match Image::from_buffer(
        &data,
        bevy::image::ImageType::Format(bevy::image::ImageFormat::Png),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::Default,
        bevy::asset::RenderAssetUsages::MAIN_WORLD | bevy::asset::RenderAssetUsages::RENDER_WORLD,
    ) {
        Ok(img) => Some(images.add(img)),
        Err(e) => {
            bevy::log::error!("Failed to decode {}: {}", path, e);
            None
        }
    }
}

pub fn all_texture_stems() -> Vec<String> {
    vec![
        "belt_east",
        "belt_north",
        "belt_turn_en",
        "miner_east",
        "miner_east_tall",
        "miner_east_2x2",
        "miner_east_3x2",
        "miner_east_3x3",
        "assembler_east",
        "turret_east",
        "storage",
        "furnace",
        "splitter_east",
        "sorter_east",
        "wall_h",
        "wall_v",
        "hq_east",
        "soldier",
        "worker",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub fn all_item_stems() -> Vec<String> {
    vec![
        "ore",
        "iron_ore",
        "copper_ore",
        "coal",
        "iron_plate",
        "copper_plate",
        "steel",
        "gear",
        "circuit",
        "ammo",
        "energy",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

// ── RenderPlugin ──

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeCache>();
        app.init_resource::<PreviewMaterials>();
        app.add_systems(Startup, setup_texture_cache);
        app.add_systems(Update, (tile_highlight, ensure_hp_bars, update_hp_bars));
        app.add_observer(spawn_projectile_visual);
        app.add_systems(
            Update,
            (
                sync_belt_slot_sprites,
                attach_enemy_visuals,
                attach_building_visuals,
                attach_unit_visuals,
                animate_belt_positions,
                wave_counter_ui,
                fps_overlay,
            ),
        );
        app.add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui);
        app.add_systems(OnExit(GameState::GameOver), despawn_game_over_ui);
    }
}

pub fn wave_counter_ui(
    wave: Res<WaveState>,
    enemies: Query<Entity, With<Enemy>>,
    peaceful: Res<PeacefulMode>,
    mut text_query: Query<(Entity, &mut Text), With<WaveCounterText>>,
    mut commands: Commands,
) {
    let msg = if peaceful.0 {
        "Peaceful Mode  |  No enemies".to_string()
    } else {
        let count = enemies.iter().len();
        format!("Wave {}  |  Enemies: {}", wave.wave, count)
    };

    if let Ok((_, mut text)) = text_query.single_mut() {
        text.0 = msg;
    } else {
        commands.spawn((
            WaveCounterText,
            Text::new(msg),
            TextFont::from_font_size(16.0),
            TextColor(Color::srgb(1.0, 0.6, 0.2)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
        ));
    }
}

pub fn spawn_game_over_ui(mut commands: Commands, wave: Res<WaveState>) {
    commands.spawn((Camera2d, GameOverUi));
    commands
        .spawn((
            GameOverUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                GameOverUi,
                Text::new("GAME OVER"),
                TextFont::from_font_size(48.0),
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));
            parent.spawn((
                GameOverUi,
                Text::new(format!("Waves survived: {}", wave.wave - 1)),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                GameOverUi,
                Text::new(""),
                TextFont::default(),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                GameOverUi,
                Text::new("Press R to restart  |  ESC for main menu"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
            ));
        });
}

pub fn despawn_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn sync_belt_slot_sprites(
    mut commands: Commands,
    textures: Res<TextureCache>,
    cfg: Res<MapConfig>,
    mut belt_query: Query<&mut BeltSlots>,
) {
    for mut bs in belt_query.iter_mut() {
        let mut to_create: Vec<(usize, ResourceId)> = Vec::new();
        for (slot_idx, item) in bs.items.iter().enumerate() {
            if item.is_some() && bs.slot_sprites[slot_idx].is_none() {
                to_create.push((slot_idx, item.as_ref().unwrap().resource_id.clone()));
            }
        }
        let mut to_destroy: Vec<usize> = Vec::new();
        for (slot_idx, sprite) in bs.slot_sprites.iter().enumerate() {
            if sprite.is_some() && bs.items[slot_idx].is_none() {
                to_destroy.push(slot_idx);
            }
        }
        for (slot_idx, resource_id) in to_create {
            let tex = textures.base(&resource_id.0);
            let target = bs.slot_positions[slot_idx];
            // Spawn at entry point so lerp animates smoothly toward target
            let entry = if slot_idx == 0 {
                let (dx, dy) = bs.direction.offset();
                let dir_vec = Vec2::new(dx as f32, dy as f32);
                target - dir_vec * (cfg.tile_size / bs.items.len() as f32)
            } else {
                bs.slot_positions[slot_idx - 1]
            };
            let entity = commands
                .spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(entry.x, entry.y, 2.5)),
                ))
                .id();
            bs.slot_sprites[slot_idx] = Some(entity);
        }
        for slot_idx in to_destroy {
            if let Some(entity) = bs.slot_sprites[slot_idx].take() {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn animate_belt_positions(
    time: Res<Time>,
    cfg: Res<MapConfig>,
    belt_query: Query<&BeltSlots>,
    mut sprite_query: Query<&mut Transform>,
) {
    let dt = time.delta_secs();
    let tile_size = cfg.tile_size;

    for bs in belt_query.iter() {
        for (slot_idx, sprite_entity) in bs.slot_sprites.iter().enumerate() {
            if let Some(entity) = sprite_entity {
                if let Ok(mut transform) = sprite_query.get_mut(*entity) {
                    let target = bs.slot_positions[slot_idx];
                    let current = Vec2::new(transform.translation.x, transform.translation.y);
                    let diff = target - current;
                    let step = bs.speed * tile_size * dt;
                    if diff.length() <= step {
                        transform.translation = Vec3::new(target.x, target.y, 2.5);
                    } else {
                        let new_pos = current + diff.normalize() * step;
                        transform.translation = Vec3::new(new_pos.x, new_pos.y, 2.5);
                    }
                }
            }
        }
    }
}

fn attach_unit_visuals(
    mut commands: Commands,
    units: Query<(Entity, &Unit, Option<&Worker>, Option<&Soldier>), Without<Sprite>>,
    textures: Res<TextureCache>,
    unit_cfg: Res<UnitConfig>,
) {
    for (entity, _unit, worker, _soldier) in units.iter() {
        let kind = if worker.is_some() {
            "worker"
        } else {
            "soldier"
        };
        let stem = unit_cfg
            .get(kind)
            .map(|d| d.texture_stem.as_str())
            .unwrap_or(kind);
        let img = textures.base(stem);
        let size = Vec2::new(48.0, 48.0);
        commands.entity(entity).insert((
            Sprite {
                image: img,
                custom_size: Some(size),
                ..default()
            },
            Visibility::default(),
        ));
        commands.entity(entity).with_children(|parent| {
            if let Some(tex) = textures.owner(stem) {
                parent.spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(size),
                        color: Color::srgb(0.2, 0.4, 0.8),
                        ..default()
                    },
                    Transform::default(),
                ));
            }
        });
    }
}

fn attach_building_visuals(
    mut commands: Commands,
    buildings: Query<(Entity, &Building), (Without<Sprite>, Without<BeltSlots>)>,
    belts: Query<(Entity, &Building, &BeltSlots), Without<Sprite>>,
    cfg: Res<MapConfig>,
    textures: Res<TextureCache>,
    registry: Res<BuildingRegistry>,
) {
    for (entity, building) in buildings.iter() {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let tw = def.tile_size.0 as f32;
        let th = def.tile_size.1 as f32;
        let size = Vec2::new(tw * cfg.tile_size, th * cfg.tile_size);
        let stem = &def.texture_stem;
        commands.entity(entity).insert((Sprite {
            image: textures.base(stem),
            custom_size: Some(size),
            ..default()
        },));
        commands.entity(entity).with_children(|parent| {
            if let Some(tex) = textures.owner(stem) {
                parent.spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(size),
                        color: Color::srgb(0.2, 0.4, 0.8),
                        ..default()
                    },
                    Transform::default(),
                ));
            }
            if let Some(tex) = textures.level(stem) {
                parent.spawn((
                    Sprite {
                        image: tex,
                        custom_size: Some(size),
                        color: Color::srgb(0.2, 0.8, 0.2),
                        ..default()
                    },
                    Transform::default(),
                ));
            }
        });
    }
    for (entity, building, _slots) in belts.iter() {
        let Some(def) = registry.get(&building.kind) else {
            continue;
        };
        let tw = def.tile_size.0 as f32;
        let th = def.tile_size.1 as f32;
        let size = Vec2::new(tw * cfg.tile_size, th * cfg.tile_size);
        let stem = &def.texture_stem;
        commands.entity(entity).insert((Sprite {
            image: textures.base(stem),
            custom_size: Some(size),
            ..default()
        },));
    }
}

fn attach_enemy_visuals(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy), Without<Mesh2d>>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    enemies_registry: Res<EnemyRegistry>,
) {
    for (entity, enemy) in enemies.iter() {
        let color = enemies_registry
            .get(&enemy.kind)
            .map(|d| d.color)
            .unwrap_or(Color::srgb(0.9, 0.2, 0.2));
        commands.entity(entity).insert((
            Mesh2d(shapes.circle.clone()),
            MeshMaterial2d(materials.add(color)),
        ));
    }
}

fn spawn_projectile_visual(
    on: On<SpawnProjectileEvent>,
    mut commands: Commands,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ev = on.event();
    commands.spawn((
        Projectile {
            target: ev.target,
            speed: ev.speed,
            damage: ev.damage,
        },
        Mesh2d(shapes.circle.clone()),
        MeshMaterial2d(materials.add(ev.color)),
        Transform::from_translation(ev.origin).with_scale(Vec3::splat(0.3)),
    ));
}

pub fn direction_arrow(dir: Direction) -> &'static str {
    match dir {
        Direction::East => ">",
        Direction::North => "^",
        Direction::West => "<",
        Direction::South => "v",
    }
}

// ── Tile highlight ──

#[derive(Component)]
struct TileHighlight;

fn tile_highlight(
    mut commands: Commands,
    build_mode: Res<BuildMode>,
    hovered: Res<HoveredTile>,
    cfg: Res<MapConfig>,
    existing: Query<Entity, With<TileHighlight>>,
    shapes: Res<ShapeCache>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    if build_mode.0.is_some() {
        return;
    }

    let Some(pos) = hovered.0 else { return };

    commands.spawn((
        TileHighlight,
        Mesh2d(shapes.square.clone()),
        MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 1.0, 0.15))),
        Transform::from_xyz(
            pos.x as f32 * cfg.tile_size,
            pos.y as f32 * cfg.tile_size,
            0.5,
        ),
    ));
}

// ── HP bars ──

fn ensure_hp_bars(
    mut commands: Commands,
    entities: Query<(Entity, &Health), (Without<HasHpBar>, Without<HpBarChild>)>,
) {
    for (entity, _health) in &entities {
        commands
            .entity(entity)
            .insert(HasHpBar)
            .with_children(|parent| {
                parent.spawn((
                    HpBarChild,
                    Sprite::from_color(Color::srgb(0.3, 1.0, 0.3), Vec2::new(24.0, 3.0)),
                    Transform::from_xyz(0.0, 20.0, 10.0),
                ));
            });
    }
}

fn update_hp_bars(
    health_q: Query<(&Health, &Children)>,
    mut sprite_q: Query<&mut Sprite, With<HpBarChild>>,
) {
    for (health, children) in health_q.iter() {
        for child in children.iter() {
            if let Ok(mut sprite) = sprite_q.get_mut(child) {
                let ratio = health.current as f32 / health.max as f32;
                let color = if ratio > 0.6 {
                    Color::srgb(0.3, 1.0, 0.3)
                } else if ratio > 0.3 {
                    Color::srgb(1.0, 0.8, 0.2)
                } else {
                    Color::srgb(1.0, 0.2, 0.2)
                };
                sprite.color = color;
                sprite.custom_size = Some(Vec2::new(24.0 * ratio, 3.0));
            }
        }
    }
}

// ── FPS overlay ──

#[derive(Component)]
pub struct FpsOverlay;

fn fps_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mut text_query: Query<(Entity, &mut Text), With<FpsOverlay>>,
    mut commands: Commands,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .map_or("--".to_string(), |v| format!("{:.0}", v));

    if let Ok((_, mut text)) = text_query.single_mut() {
        text.0 = format!("FPS: {}", fps);
    } else {
        commands.spawn((
            FpsOverlay,
            Text::new(format!("FPS: {}", fps)),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
        ));
    }
}
