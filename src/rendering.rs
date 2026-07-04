use std::collections::HashMap;
use bevy::prelude::*;
use crate::economy::components::{BuildMode, Direction, HpBarChild, HasHpBar};
use crate::enemy::components::Health;
use crate::map::components::HoveredTile;
use crate::map::config::MapConfig;

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

fn setup_texture_cache(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let building_stems = all_texture_stems();
    let item_stems = all_item_stems();
    let total = building_stems.len() + item_stems.len();
    let mut base = HashMap::with_capacity(total);
    let mut owner = HashMap::with_capacity(total);
    let mut level = HashMap::with_capacity(total);

    for stem in &building_stems {
        let s = stem.as_str();
        base.insert(stem.clone(), load_png(&mut images, s, "base").unwrap_or_default());
        owner.insert(stem.clone(), load_png(&mut images, s, "owner"));
        level.insert(stem.clone(), load_png(&mut images, s, "level"));
    }

    for stem in &item_stems {
        let s = stem.as_str();
        base.insert(stem.clone(), load_png(&mut images, s, "base").unwrap_or_default());
    }

    commands.insert_resource(TextureCache { base, owner, level });
}

fn load_png(
    images: &mut Assets<Image>,
    stem: &str,
    layer: &str,
) -> Option<Handle<Image>> {
    let path = format!("assets/textures/{}_{}.png", stem, layer);
    let data = std::fs::read(&path).ok()?;
    match Image::from_buffer(
        &data,
        bevy::image::ImageType::Format(bevy::image::ImageFormat::Png),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::Default,
        bevy::asset::RenderAssetUsages::MAIN_WORLD
            | bevy::asset::RenderAssetUsages::RENDER_WORLD,
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
        "belt_east", "belt_north", "belt_turn_en",
        "miner_east", "miner_east_tall", "miner_east_2x2", "miner_east_3x2", "miner_east_3x3",
        "assembler_east", "turret_east", "storage", "furnace",
        "splitter_east", "sorter_east",
        "wall_h", "wall_v", "hq_east",
        "soldier", "worker",
    ].into_iter().map(String::from).collect()
}

/// All item texture stems (used for belt items).
pub fn all_item_stems() -> Vec<String> {
    vec![
        "ore", "iron_ore", "copper_ore", "coal",
        "iron_plate", "copper_plate", "steel",
        "gear", "circuit", "ammo", "energy",
    ].into_iter().map(String::from).collect()
}

/// Map a ResourceId to its item texture stem.
pub fn item_stem(id: &str) -> &str {
    id
}

/// Map a BuildingDef id to its texture stem.
pub fn texture_stem(id: &str) -> &str {
    match id {
        "storage" => "storage",
        "belt" => "belt_east",
        "splitter" => "splitter_east",
        "sorter" => "sorter_east",
        "miner" => "miner_east",
        "assembler" => "assembler_east",
        "turret" => "turret_east",
        "wall" => "wall_h",
        "hq" => "hq_east",
        "soldier" => "soldier",
        "worker" => "worker",
        _ => id,
    }
}

// ── RenderPlugin ──

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeCache>();
        app.init_resource::<PreviewMaterials>();
        app.add_systems(Startup, setup_texture_cache);
        app.add_systems(Update, (
            tile_highlight,
            ensure_hp_bars,
            update_hp_bars,
        ));
    }
}

pub fn material_from_color(
    materials: &mut Assets<ColorMaterial>,
    color: Color,
) -> Handle<ColorMaterial> {
    materials.add(color)
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
        commands.entity(entity).insert(HasHpBar).with_children(|parent| {
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
