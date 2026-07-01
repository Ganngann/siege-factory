use bevy::prelude::*;
use crate::economy::systems::Direction;
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
            diamond: meshes.add(Rectangle::new(s, s)),
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

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeCache>();
    }
}

pub fn material_from_color(
    materials: &mut Assets<ColorMaterial>,
    color: Color,
) -> Handle<ColorMaterial> {
    materials.add(ColorMaterial::from_color(color))
}

pub fn direction_arrow(dir: Direction) -> &'static str {
    match dir {
        Direction::East => ">",
        Direction::North => "^",
        Direction::West => "<",
        Direction::South => "v",
    }
}
