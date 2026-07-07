use crate::core::modding::ModRegistry;
use crate::economy::building::BuildingRegistry;
use crate::economy::resource::ResourceRegistry;
use crate::enemy::registry::EnemyRegistry;
use crate::map::config::MapConfig;
use bevy::prelude::*;
use std::collections::HashMap;

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

#[derive(Resource)]
pub struct PreviewMaterials {
    pub deconstruct_building: Handle<ColorMaterial>,
    pub deconstruct_zone: Handle<ColorMaterial>,
    pub build_valid: Handle<ColorMaterial>,
    pub build_invalid: Handle<ColorMaterial>,
    pub indicator_input: Handle<ColorMaterial>,
    pub indicator_output: Handle<ColorMaterial>,
    pub fog: Handle<ColorMaterial>,
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
            fog: materials.add(Color::srgba(0.0, 0.0, 0.0, 1.0)),
        }
    }
}

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

pub fn setup_texture_cache(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    building_registry: Res<BuildingRegistry>,
    resource_registry: Res<ResourceRegistry>,
    enemy_registry: Res<EnemyRegistry>,
    mods: Res<ModRegistry>,
) {
    let building_stems = collect_building_stems(&building_registry);
    let item_stems = collect_item_stems(&resource_registry);
    let enemy_stems = collect_enemy_stems(&enemy_registry);
    let total = building_stems.len() + item_stems.len() + enemy_stems.len();
    let mut base = HashMap::with_capacity(total);
    let mut owner = HashMap::with_capacity(total);
    let mut level = HashMap::with_capacity(total);

    for stem in &building_stems {
        let s = stem.as_str();
        base.insert(
            stem.clone(),
            load_png(&mut images, &mods, s, "base").unwrap_or_default(),
        );
        owner.insert(stem.clone(), load_png(&mut images, &mods, s, "owner"));
        level.insert(stem.clone(), load_png(&mut images, &mods, s, "level"));
    }

    for stem in &item_stems {
        let s = stem.as_str();
        base.insert(
            stem.clone(),
            load_png(&mut images, &mods, s, "base").unwrap_or_default(),
        );
    }

    for stem in &enemy_stems {
        let s = stem.as_str();
        base.insert(
            stem.clone(),
            load_png(&mut images, &mods, s, "base").unwrap_or_default(),
        );
    }

    commands.insert_resource(TextureCache { base, owner, level });
}

fn load_png(
    images: &mut Assets<Image>,
    mods: &ModRegistry,
    stem: &str,
    layer: &str,
) -> Option<Handle<Image>> {
    let data = if let Some(mod_data) = mods.load_texture(stem, layer) {
        mod_data
    } else {
        let path = format!("assets/textures/{}_{}.png", stem, layer);
        std::fs::read(&path).ok()?
    };
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
            bevy::log::error!("Failed to decode texture {} {}: {}", stem, layer, e);
            None
        }
    }
}

fn collect_building_stems(registry: &BuildingRegistry) -> Vec<String> {
    let extra = [
        "belt_north",
        "belt_turn_en",
        "miner_east_tall",
        "miner_east_2x2",
        "miner_east_3x2",
        "miner_east_3x3",
        "wall_v",
        "soldier",
        "worker",
        "cultivator",
        "player",
        "builder",
    ];
    let mut stems: Vec<String> = registry
        .buildings
        .iter()
        .flat_map(|b| {
            let mut s = vec![b.texture_stem.clone()];
            s.extend(b.tiers.iter().map(|t| t.texture.clone()));
            s
        })
        .collect();
    stems.extend(extra.iter().map(|&s| s.to_string()));
    stems
}

fn collect_item_stems(registry: &ResourceRegistry) -> Vec<String> {
    registry.resources.keys().cloned().collect()
}

fn collect_enemy_stems(registry: &EnemyRegistry) -> Vec<String> {
    registry.enemies.keys().cloned().collect()
}
