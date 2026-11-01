//! Turns the shape of the ground into a mesh, painted with soft patches of color.

use crate::Forest;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use movement::WalkSurface;
use std::sync::Arc;

/// The distance between neighboring points of the ground mesh.
const CELL_SIZE: f32 = 1.0;
/// How big a patch of moss or grass usually is.
const PATCH_SIZE: f32 = 7.0;
/// Smaller mottling inside each patch, so it never reads as one flat color.
const MOTTLE_SIZE: f32 = 1.6;
const MOTTLE_WEIGHT: f32 = 0.45;
/// How soft the edge of the trail is, as a share of its width.
const TRAIL_FADE: f32 = 1.0;
const GROUND_ROUGHNESS: f32 = 0.95;
const GROUND_REFLECTANCE: f32 = 0.2;

/// The colors the ground is painted with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundPalette {
    pub moss: Color,
    pub grass: Color,
    pub dirt: Color,
}

pub(super) struct GroundPlugin {
    pub forest: Arc<Forest>,
    pub palette: GroundPalette,
}

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        let forest = self.forest.clone();
        let palette = self.palette;
        app.add_systems(
            Startup,
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>| {
                commands.spawn((
                    Mesh3d(meshes.add(ground_mesh(&forest, palette))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        perceptual_roughness: GROUND_ROUGHNESS,
                        reflectance: GROUND_REFLECTANCE,
                        ..default()
                    })),
                ));
            },
        );
    }
}

fn ground_mesh(forest: &Forest, palette: GroundPalette) -> Mesh {
    let reach = forest.config().half_extent + forest.config().margin;
    let side = (reach * 2.0 / CELL_SIZE) as u32 + 1;
    let point = |x: u32, z: u32| Vec2::new(x as f32, z as f32) * CELL_SIZE - Vec2::splat(reach);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    for z in 0..side {
        for x in 0..side {
            let at = point(x, z);
            positions.push([at.x, forest.ground_height(at), at.y]);
            normals.push(surface_normal(forest, at).to_array());
            colors.push(paint(forest, palette, at).to_f32_array());
        }
    }

    let mut indices = Vec::new();
    for z in 0..side - 1 {
        for x in 0..side - 1 {
            let corner = z * side + x;
            let (right, below) = (corner + 1, corner + side);
            indices.extend([corner, below, right, right, below, below + 1]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}

fn surface_normal(forest: &Forest, at: Vec2) -> Vec3 {
    let across = forest.ground_height(at + Vec2::X * CELL_SIZE)
        - forest.ground_height(at - Vec2::X * CELL_SIZE);
    let along = forest.ground_height(at + Vec2::Y * CELL_SIZE)
        - forest.ground_height(at - Vec2::Y * CELL_SIZE);
    Vec3::new(-across, 2.0 * CELL_SIZE, -along).normalize()
}

fn paint(forest: &Forest, palette: GroundPalette, at: Vec2) -> LinearRgba {
    let moss: LinearRgba = palette.moss.into();
    let grass: LinearRgba = palette.grass.into();
    let dirt: LinearRgba = palette.dirt.into();

    let patch = forest.patchiness(at / PATCH_SIZE);
    let mottle = forest.patchiness(at / MOTTLE_SIZE) - 0.5;
    let growth = moss.mix(&grass, (patch + mottle * MOTTLE_WEIGHT).clamp(0.0, 1.0));
    let width = forest.config().trail_width;
    let bare = 1.0 - ((forest.trail_distance(at) - width) / (width * TRAIL_FADE)).clamp(0.0, 1.0);
    growth.mix(&dirt, bare)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorldConfig;
    use bdd::spec;
    use bevy::mesh::VertexAttributeValues;

    const PALETTE: GroundPalette = GroundPalette {
        moss: Color::srgb(0.1, 0.3, 0.1),
        grass: Color::srgb(0.2, 0.4, 0.1),
        dirt: Color::srgb(0.3, 0.2, 0.1),
    };

    fn forest() -> Forest {
        Forest::new(WorldConfig {
            seed: 7,
            half_extent: 10.0,
            margin: 2.0,
            clearing_radius: 5.0,
            hill_height: 1.5,
            hill_spacing: 8.0,
            rim_height: 3.0,
            trail_radius: 8.0,
            trail_width: 1.0,
            plantings: Vec::new(),
        })
    }

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => panic!("the mesh has positions"),
        }
    }

    #[spec]
    fn ground_painting() {
        given("a forest", || {
            let forest = forest();

            when("its mesh is built", || {
                let mesh = ground_mesh(&forest, PALETTE);

                test("it covers the world and its margin", || {
                    let furthest = positions(&mesh)
                        .iter()
                        .fold(0.0_f32, |far, point| far.max(point[0].abs()));
                    assert_eq!(furthest, 12.0);
                });

                test("every point sits at the ground's height", || {
                    for point in positions(&mesh) {
                        let height = forest.ground_height(Vec2::new(point[0], point[2]));
                        assert_eq!(point[1], height);
                    }
                });

                test("its triangles face upward", || {
                    let points = positions(&mesh);
                    let Some(Indices::U32(indices)) = mesh.indices() else {
                        panic!("the mesh has indices");
                    };
                    for triangle in indices.chunks(3) {
                        let [a, b, c] = [0, 1, 2].map(|i| Vec3::from(points[triangle[i] as usize]));
                        assert!((b - a).cross(c - a).y > 0.0);
                    }
                });
            });

            when("the trail is painted", || {
                let on_trail = paint(&forest, PALETTE, Vec2::new(0.0, -3.0));

                test("it is bare dirt", || {
                    assert_eq!(on_trail, LinearRgba::from(PALETTE.dirt));
                });
            });

            when("ground away from the trail is painted", || {
                let colors: Vec<LinearRgba> = (0..20)
                    .map(|i| paint(&forest, PALETTE, Vec2::new(i as f32 * 3.0, 40.0)))
                    .collect();

                test("the color varies from patch to patch", || {
                    assert!(colors.iter().any(|color| *color != colors[0]));
                });
            });

            when("level ground's normal is measured", || {
                let normal = surface_normal(&forest, Vec2::ZERO);

                test("it points straight up", || {
                    assert_eq!(normal, Vec3::Y);
                });
            });
        });

        given("an app with the ground plugin", || {
            let mut app = App::new();
            app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
            app.init_asset::<Mesh>();
            app.init_asset::<StandardMaterial>();
            app.add_plugins(GroundPlugin {
                forest: Arc::new(forest()),
                palette: PALETTE,
            });

            when("it starts", || {
                app.update();
                let mut ground = app.world_mut().query::<&Mesh3d>();

                test("one ground mesh exists", || {
                    assert_eq!(ground.iter(app.world()).count(), 1);
                });
            });
        });
    }
}
