use bevy::prelude::*;
use crate::economy::components::{BuildMode, Direction, HpBarChild, HasHpBar};
use crate::enemy::components::Health;
use crate::map::components::HoveredTile;
use crate::map::config::MapConfig;

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

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeCache>();
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
