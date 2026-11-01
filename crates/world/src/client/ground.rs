//! Turns the shape of the ground into a mesh. A tiled texture gives it surface detail, and
//! soft patches of color tint that texture into moss, grass, and trail.

use crate::Forest;
use bevy::asset::RenderAssetUsages;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
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
const GROUND_REFLECTANCE: f32 = 0.2;
/// Keeps a texture sharp when the ground is seen at a low angle.
const TEXTURE_ANISOTROPY: u16 = 8;

/// The colors the ground's texture is tinted with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundPalette {
    pub moss: Color,
    pub grass: Color,
    pub dirt: Color,
}

/// The texture covering the ground. Its paths are relative to the asset folder.
#[derive(Clone, Debug, PartialEq)]
pub struct GroundTexture {
    pub color: String,
    pub normal: String,
    pub roughness: String,
    /// How many world units one copy of the texture covers.
    pub tile_size: f32,
}

pub(super) struct GroundPlugin {
    pub forest: Arc<Forest>,
    pub palette: GroundPalette,
    pub texture: GroundTexture,
}

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        let forest = self.forest.clone();
        let palette = self.palette;
        let texture = self.texture.clone();
        app.add_systems(
            Startup,
            move |mut commands: Commands,
                  assets: Res<AssetServer>,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut materials: ResMut<Assets<StandardMaterial>>| {
                commands.spawn((
                    Mesh3d(meshes.add(ground_mesh(&forest, palette, texture.tile_size))),
                    MeshMaterial3d(materials.add(ground_material(&assets, &texture))),
                ));
            },
        );
    }
}

fn ground_material(assets: &AssetServer, texture: &GroundTexture) -> StandardMaterial {
    let load = |path: &str, is_srgb: bool| {
        assets
            .load_builder()
            .with_settings(move |settings: &mut ImageLoaderSettings| {
                repeat_texture(settings, is_srgb);
            })
            .load(path.to_owned())
    };
    StandardMaterial {
        base_color_texture: Some(load(&texture.color, true)),
        normal_map_texture: Some(load(&texture.normal, false)),
        metallic_roughness_texture: Some(load(&texture.roughness, false)),
        // The roughness map supplies the real value, and this scales it.
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: GROUND_REFLECTANCE,
        ..default()
    }
}

/// Lets a texture tile across the ground. Normal and roughness maps hold data, not colors,
/// so they aren't stored as sRGB.
fn repeat_texture(settings: &mut ImageLoaderSettings, is_srgb: bool) {
    settings.is_srgb = is_srgb;
    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: TEXTURE_ANISOTROPY,
        ..default()
    });
}

fn ground_mesh(forest: &Forest, palette: GroundPalette, tile_size: f32) -> Mesh {
    let reach = forest.config().half_extent + forest.config().margin;
    let side = (reach * 2.0 / CELL_SIZE) as u32 + 1;
    let point = |x: u32, z: u32| Vec2::new(x as f32, z as f32) * CELL_SIZE - Vec2::splat(reach);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut texture_coordinates = Vec::new();
    for z in 0..side {
        for x in 0..side {
            let at = point(x, z);
            positions.push([at.x, forest.ground_height(at), at.y]);
            normals.push(surface_normal(forest, at).to_array());
            colors.push(paint(forest, palette, at).to_f32_array());
            texture_coordinates.push((at / tile_size).to_array());
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, texture_coordinates)
    .with_inserted_indices(Indices::U32(indices))
    .with_generated_tangents()
    .expect("the ground mesh has everything tangents need")
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
    const TILE_SIZE: f32 = 4.0;

    fn texture() -> GroundTexture {
        GroundTexture {
            color: "color.jpg".into(),
            normal: "normal.jpg".into(),
            roughness: "roughness.jpg".into(),
            tile_size: TILE_SIZE,
        }
    }

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

    fn texture_coordinates(mesh: &Mesh) -> &[[f32; 2]] {
        match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            Some(VertexAttributeValues::Float32x2(coordinates)) => coordinates,
            _ => panic!("the mesh has texture coordinates"),
        }
    }

    fn sampler_of(settings: &ImageLoaderSettings) -> &ImageSamplerDescriptor {
        match &settings.sampler {
            ImageSampler::Descriptor(descriptor) => descriptor,
            ImageSampler::Default => panic!("the sampler was configured"),
        }
    }

    #[spec]
    fn ground_painting() {
        given("a forest", || {
            let forest = forest();

            when("its mesh is built", || {
                let mesh = ground_mesh(&forest, PALETTE, TILE_SIZE);

                test("every point has texture coordinates", || {
                    let (points, coordinates) = (positions(&mesh), texture_coordinates(&mesh));
                    assert_eq!(points.len(), coordinates.len());
                });

                test("the texture repeats every tile", || {
                    for (point, coordinate) in
                        positions(&mesh).iter().zip(texture_coordinates(&mesh))
                    {
                        assert_eq!(coordinate[0], point[0] / TILE_SIZE);
                        assert_eq!(coordinate[1], point[2] / TILE_SIZE);
                    }
                });

                test("every point has a tangent for the normal map", || {
                    let tangents = mesh.attribute(Mesh::ATTRIBUTE_TANGENT).unwrap();
                    assert_eq!(tangents.len(), positions(&mesh).len());
                });

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

            when("the trail and the ground beside it are painted", || {
                let on_trail = paint(&forest, PALETTE, Vec2::new(0.0, -3.0));
                let beside_it = paint(&forest, PALETTE, Vec2::new(4.0, -3.0));

                test("their tints still differ", || {
                    assert_ne!(on_trail, beside_it);
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
            app.init_asset::<Image>();
            app.add_plugins(GroundPlugin {
                forest: Arc::new(forest()),
                palette: PALETTE,
                texture: texture(),
            });

            when("it starts", || {
                app.update();
                let mut ground = app.world_mut().query::<&Mesh3d>();
                let mut surfaces = app.world_mut().query::<&MeshMaterial3d<StandardMaterial>>();
                let handle = surfaces.single(app.world()).unwrap().0.clone();
                let materials = app.world().resource::<Assets<StandardMaterial>>();
                let material = materials.get(&handle).unwrap();

                test("one ground mesh exists", || {
                    assert_eq!(ground.iter(app.world()).count(), 1);
                });

                test("the material uses every map", || {
                    assert!(material.base_color_texture.is_some());
                    assert!(material.normal_map_texture.is_some());
                    assert!(material.metallic_roughness_texture.is_some());
                });
            });
        });

        given("a texture's loader settings", || {
            let mut settings = ImageLoaderSettings::default();

            when("it's set up as a color map", || {
                repeat_texture(&mut settings, true);

                test("it repeats in both directions", || {
                    let sampler = sampler_of(&settings);
                    assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
                    assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
                });

                test("it stays in sRGB", || {
                    assert!(settings.is_srgb);
                });
            });

            when("it's set up as a data map", || {
                repeat_texture(&mut settings, false);

                test("it isn't treated as a color", || {
                    assert!(!settings.is_srgb);
                });
            });
        });
    }
}
