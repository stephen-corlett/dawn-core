//! The look development scene: shows art under the game's real mood, at fixed distances, in sun
//! and in shadow. See docs/art-pipeline.md.
//!
//! `just lookdev` opens it, and the arrow keys, 1 to 3, and Tab change the view.
//! `just lookdev --capture <folder>` saves every view, and then exits. Name models after the
//! options to show those instead of the hero assets.

use bevy::camera::primitives::Aabb;
use bevy::ecs::system::SystemParam;
use bevy::gilrs::GilrsPlugin;
use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowResolution;
use clap::Parser;
use client::config::ClientConfig;
use client::look::{avatar_plugin, world_plugin};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use world::{Forest, WorldConfig};

const HEROES: [&str; 3] = [CHARACTER, "nature/Pine_1.gltf", "nature/Rock_Medium_1.gltf"];
/// Stands for every part of the player's character.
const CHARACTER: &str = "character";

/// How far the camera stands from the surface it faces, in meters.
const DISTANCES: [f32; 3] = [1.0, 5.0, 20.0];
const WINDOW_SIZE: (u32, u32) = (1280, 720);
/// A person's eye height, which is what the camera aims at on anything tall enough.
const EYE_HEIGHT: f32 = 1.6;
/// How far up anything shorter is aimed at. High enough to show a character's face.
const FOCUS_SHARE_OF_HEIGHT: f32 = 0.85;
const STATION_SPACING: f32 = 14.0;
/// How far the sunlit and shaded rows sit to either side of the world's center.
const ROW_OFFSET: f32 = 55.0;
/// The wall that shades one row stands up-sun of it, behind the camera in every view.
const WALL_DISTANCE: f32 = 30.0;
const WALL_SIZE: Vec3 = Vec3::new(120.0, 30.0, 2.0);
/// Long enough for the sky's light and the post-processing to settle after loading.
const SETTLE_SECS: f32 = 3.0;
const FRAMES_PER_VIEW: u32 = 15;
/// Long enough for the last screenshot to reach the disk.
const FINISH_FRAMES: u32 = 120;

#[derive(Parser)]
struct Args {
    /// Save every view to this folder, and then exit.
    #[arg(long)]
    capture: Option<PathBuf>,
    /// Models to show, relative to the assets folder. Defaults to the hero assets.
    models: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Row {
    Sunlit,
    Shaded,
}

impl Row {
    fn name(self) -> &'static str {
        match self {
            Row::Sunlit => "sun",
            Row::Shaded => "shadow",
        }
    }

    fn center(self, across: Vec3) -> Vec3 {
        match self {
            Row::Sunlit => across * -ROW_OFFSET,
            Row::Shaded => across * ROW_OFFSET,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct View {
    station: usize,
    row: Row,
    distance: usize,
}

struct Station {
    name: String,
    sunlit: Vec<Entity>,
    shaded: Vec<Entity>,
    scenes: Vec<Handle<WorldAsset>>,
}

impl Station {
    fn roots(&self, row: Row) -> &[Entity] {
        match row {
            Row::Sunlit => &self.sunlit,
            Row::Shaded => &self.shaded,
        }
    }
}

/// Where the sun is, flat along the ground, and the direction that runs across it.
#[derive(Clone, Copy)]
struct Compass {
    towards_sun: Vec3,
    across: Vec3,
}

impl Compass {
    fn new(sun: Vec3) -> Self {
        let towards_sun = Vec3::new(sun.x, 0.0, sun.z).normalize();
        Self {
            towards_sun,
            across: towards_sun.cross(Vec3::Y).normalize(),
        }
    }

    /// From the sun's side, so that lit surfaces face the camera, and a little to one side,
    /// so that forms show.
    fn to_camera(self) -> Vec3 {
        (self.towards_sun * 0.8 + self.across * 0.55 + Vec3::Y * 0.18).normalize()
    }
}

enum Stage {
    Loading,
    Settling { until: f32 },
    Viewing { drawn: Option<View> },
    Capturing { next: usize, frames: u32 },
    Finishing { frames: u32 },
}

#[derive(Resource)]
struct Scene {
    stations: Vec<Station>,
    compass: Compass,
    capture: Option<PathBuf>,
    stage: Stage,
    view: View,
}

fn main() -> AppExit {
    let args = Args::parse();
    let config = ClientConfig::dev_default();
    let mut models = args.models;
    if models.is_empty() {
        models = HEROES.map(String::from).to_vec();
    }

    let forest = Arc::new(Forest::new(WorldConfig {
        seed: 1,
        half_extent: 150.0,
        margin: 20.0,
        clearing_radius: 140.0,
        hill_height: 0.0,
        hill_spacing: 15.0,
        rim_height: 0.0,
        trail_radius: 400.0,
        trail_width: 0.1,
        plantings: vec![],
    }));
    let world = world_plugin(forest);
    let compass = Compass::new(world.mood.sun_position);

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Look development".into(),
                    resolution: WindowResolution::new(WINDOW_SIZE.0, WINDOW_SIZE.1),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: config.asset_folder.clone(),
                ..default()
            })
            .disable::<GilrsPlugin>(),
    );
    app.add_plugins(world);
    app.insert_resource(Scene {
        stations: Vec::new(),
        compass,
        capture: args.capture,
        stage: Stage::Loading,
        view: View {
            station: 0,
            row: Row::Sunlit,
            distance: 1,
        },
    });
    app.add_systems(
        Startup,
        move |mut commands: Commands,
              assets: Res<AssetServer>,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<StandardMaterial>>,
              mut scene: ResMut<Scene>| {
            commands.spawn((Camera3d::default(), Transform::default()));
            spawn_wall(&mut commands, &mut meshes, &mut materials, compass);
            scene.stations = spawn_stations(&mut commands, &assets, compass, &models);
        },
    );
    app.add_systems(Update, (wait_for_assets, browse_views, capture_views));
    app.run()
}

/// Casts the shade over one row. The camera always faces away from the sun, so it never sees this.
fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    compass: Compass,
) {
    let at = Row::Shaded.center(compass.across) + compass.towards_sun * WALL_DISTANCE;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(WALL_SIZE))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_translation(at + Vec3::Y * WALL_SIZE.y * 0.5)
            .looking_to(compass.towards_sun, Vec3::Y),
    ));
}

fn spawn_stations(
    commands: &mut Commands,
    assets: &AssetServer,
    compass: Compass,
    models: &[String],
) -> Vec<Station> {
    let middle = (models.len() as f32 - 1.0) * 0.5;
    models
        .iter()
        .enumerate()
        .map(|(index, model)| {
            let parts = if model == CHARACTER {
                avatar_plugin().appearance.parts
            } else {
                vec![model.clone()]
            };
            let scenes: Vec<Handle<WorldAsset>> = parts
                .iter()
                .map(|part| assets.load(GltfAssetLabel::Scene(0).from_asset(part.clone())))
                .collect();
            let along_row = (index as f32 - middle) * STATION_SPACING;
            let mut spawn_row = |row: Row| -> Vec<Entity> {
                let at = row.center(compass.across) + compass.across * along_row;
                scenes
                    .iter()
                    .map(|scene| {
                        commands
                            .spawn((
                                WorldAssetRoot(scene.clone()),
                                Transform::from_translation(at),
                            ))
                            .id()
                    })
                    .collect()
            };
            let sunlit = spawn_row(Row::Sunlit);
            let shaded = spawn_row(Row::Shaded);
            Station {
                name: model.clone(),
                sunlit,
                shaded,
                scenes,
            }
        })
        .collect()
}

#[derive(SystemParam)]
struct Framing<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    meshes: Query<'w, 's, (&'static Aabb, &'static GlobalTransform)>,
    ray_cast: MeshRayCast<'w, 's>,
}

fn wait_for_assets(
    mut scene: ResMut<Scene>,
    time: Res<Time>,
    assets: Res<AssetServer>,
    framing: Framing,
    mut exit: MessageWriter<AppExit>,
) {
    let scene = &mut *scene;
    match scene.stage {
        Stage::Loading => {
            let scenes = || scene.stations.iter().flat_map(|station| &station.scenes);
            if scenes().any(|handle| assets.load_state(handle).is_failed()) {
                error!("A model failed to load");
                exit.write(AppExit::error());
                return;
            }
            let has_meshes = |roots: &[Entity]| {
                !mesh_entities(roots, &framing.children, &framing.meshes).is_empty()
            };
            let loaded = scenes().all(|handle| assets.is_loaded_with_dependencies(handle))
                && scene.stations.iter().all(|station| {
                    has_meshes(station.roots(Row::Sunlit)) && has_meshes(station.roots(Row::Shaded))
                });
            if loaded && !scene.stations.is_empty() {
                scene.stage = Stage::Settling {
                    until: time.elapsed_secs() + SETTLE_SECS,
                };
            }
        }
        Stage::Settling { until } if time.elapsed_secs() >= until => {
            scene.stage = if scene.capture.is_some() {
                Stage::Capturing { next: 0, frames: 0 }
            } else {
                Stage::Viewing { drawn: None }
            };
        }
        _ => {}
    }
}

fn browse_views(
    mut scene: ResMut<Scene>,
    keys: Res<ButtonInput<KeyCode>>,
    mut framing: Framing,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    mut window: Single<&mut Window>,
) {
    let scene = &mut *scene;
    let Stage::Viewing { drawn } = &mut scene.stage else {
        return;
    };
    let view = &mut scene.view;
    let stations = scene.stations.len();
    if keys.just_pressed(KeyCode::ArrowRight) {
        view.station = (view.station + 1) % stations;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        view.station = (view.station + stations - 1) % stations;
    }
    for (distance, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]
        .into_iter()
        .enumerate()
    {
        if keys.just_pressed(key) {
            view.distance = distance;
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        view.row = match view.row {
            Row::Sunlit => Row::Shaded,
            Row::Shaded => Row::Sunlit,
        };
    }
    if *drawn != Some(*view) {
        *drawn = Some(*view);
        window.title = format!("Look development: {}", view_name(&scene.stations, *view));
        **camera = camera_for(*view, &scene.stations, scene.compass, &mut framing);
    }
}

fn capture_views(
    mut scene: ResMut<Scene>,
    mut framing: Framing,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let scene = &mut *scene;
    let views = all_views(scene.stations.len());
    match &mut scene.stage {
        Stage::Capturing { next, frames } => {
            let Some(view) = views.get(*next).copied() else {
                scene.stage = Stage::Finishing { frames: 0 };
                return;
            };
            if *frames == 0 {
                **camera = camera_for(view, &scene.stations, scene.compass, &mut framing);
            }
            *frames += 1;
            if *frames == FRAMES_PER_VIEW {
                let folder = scene.capture.as_ref().unwrap();
                std::fs::create_dir_all(folder).unwrap();
                let name = view_name(&scene.stations, view);
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(folder.join(format!("{name}.png"))));
                *next += 1;
                *frames = 0;
            }
        }
        Stage::Finishing { frames } => {
            *frames += 1;
            if *frames < FINISH_FRAMES {
                return;
            }
            let folder = scene.capture.as_ref().unwrap();
            let saved = |view: &&View| {
                let name = view_name(&scene.stations, **view);
                folder.join(format!("{name}.png")).exists()
            };
            let count = views.iter().filter(saved).count();
            if count == views.len() {
                println!("Saved {count} views to {}", folder.display());
                exit.write(AppExit::Success);
            } else {
                error!("Only {count} of {} views were saved", views.len());
                exit.write(AppExit::error());
            }
        }
        _ => {}
    }
}

fn all_views(stations: usize) -> Vec<View> {
    let mut views = Vec::new();
    for station in 0..stations {
        for row in [Row::Sunlit, Row::Shaded] {
            for distance in 0..DISTANCES.len() {
                views.push(View {
                    station,
                    row,
                    distance,
                });
            }
        }
    }
    views
}

fn view_name(stations: &[Station], view: View) -> String {
    let model = Path::new(&stations[view.station].name);
    let stem = model.file_stem().unwrap().to_string_lossy();
    let meters = DISTANCES[view.distance] as u32;
    format!("{stem}_{}_{meters}m", view.row.name())
}

/// Every mesh under these roots.
fn mesh_entities(
    roots: &[Entity],
    children: &Query<&Children>,
    meshes: &Query<(&Aabb, &GlobalTransform)>,
) -> HashSet<Entity> {
    roots
        .iter()
        .flat_map(|root| children.iter_descendants(*root))
        .filter(|entity| meshes.contains(*entity))
        .collect()
}

/// The box around these meshes in the world.
fn world_bounds(
    entities: &HashSet<Entity>,
    meshes: &Query<(&Aabb, &GlobalTransform)>,
) -> (Vec3, Vec3) {
    let mut low = Vec3::MAX;
    let mut high = Vec3::MIN;
    for (aabb, transform) in entities
        .iter()
        .filter_map(|entity| meshes.get(*entity).ok())
    {
        let center = Vec3::from(aabb.center);
        let half = Vec3::from(aabb.half_extents);
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let corner = transform.transform_point(center + half * Vec3::new(x, y, z));
                    low = low.min(corner);
                    high = high.max(corner);
                }
            }
        }
    }
    (low, high)
}

/// Stands the camera at the view's distance from the surface it faces. Distance from a model's
/// bounding box would be wrong, because a tree's trunk sits meters inside its branches.
fn camera_for(
    view: View,
    stations: &[Station],
    compass: Compass,
    framing: &mut Framing,
) -> Transform {
    let roots = stations[view.station].roots(view.row);
    let entities = mesh_entities(roots, &framing.children, &framing.meshes);
    let (low, high) = world_bounds(&entities, &framing.meshes);
    let middle = (low + high) * 0.5;
    let height = (high.y - low.y) * FOCUS_SHARE_OF_HEIGHT;
    let focus = Vec3::new(middle.x, low.y + height.min(EYE_HEIGHT), middle.z);
    let to_camera = compass.to_camera();
    let reach = (high - low).length() + 1.0;
    let filter = |entity: Entity| entities.contains(&entity);
    let settings = MeshRayCastSettings::default()
        .with_visibility(RayCastVisibility::Any)
        .with_filter(&filter)
        .never_early_exit();
    let ray = Ray3d::new(focus + to_camera * reach, Dir3::new(-to_camera).unwrap());
    let surface = framing
        .ray_cast
        .cast_ray(ray, &settings)
        .iter()
        .map(|(_, hit)| hit.point)
        .min_by(|a, b| {
            a.distance_squared(ray.origin)
                .total_cmp(&b.distance_squared(ray.origin))
        })
        .unwrap_or(focus);
    let eye = surface + to_camera * DISTANCES[view.distance];
    Transform::from_translation(eye).looking_at(focus, Vec3::Y)
}
